use anyhow::Result;
use facade::component::anything_item::Something;
use jwalk::{WalkDir, WalkDirGeneric};
use once_cell::sync::Lazy;
use std::fs;
use std::sync::Mutex;
use tantivy::{
    Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, TantivyError, Term,
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{
        Field, INDEXED, IndexRecordOption, STORED, Schema, TextFieldIndexing, TextOptions, Value,
    },
    tokenizer::{SimpleTokenizer, TokenStream, Tokenizer},
};
use tantivy_jieba::JiebaTokenizer;
use time::OffsetDateTime;
use tracing::{debug, error};
use vaultify::VAULTIFY;
use whichlang::{Lang, detect_language};

pub static TANTIVY_INDEX: Lazy<TantivyIndex> = Lazy::new(|| {
    debug!("initializing tantivy");
    TantivyIndex::new().unwrap()
});

pub struct TantivyIndex {
    // schema: Schema,
    name_field: Field,
    path_field: Field,
    index: Index,
    index_writer: Mutex<IndexWriter>,
    index_reader: IndexReader,
}

impl TantivyIndex {
    pub fn new() -> Result<Self, TantivyError> {
        debug!("building initial tantivy index...");
        let mut schema_builder = Schema::builder();

        let name_options = TextOptions::default().set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("mixed")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        );
        let name_field = schema_builder.add_text_field("name", name_options);
        let path_field = schema_builder.add_bytes_field("path", INDEXED | STORED);
        // schema_builder.add_bytes_field("if_folder", STORED);
        let schema = schema_builder.build();

        let index_path = VAULTIFY.get("tantivy_path").unwrap();
        let index = match Index::open_in_dir(&index_path) {
            Ok(index) => index,
            Err(_) => {
                if let Err(e) = fs::create_dir_all(&index_path) {
                    error!("Failed to create directory at {}: {}", &index_path, e);
                    panic!("Failed to create directory at {}: {}", &index_path, e);
                }
                match Index::create_in_dir(&index_path, schema.clone()) {
                    Ok(index) => {
                        debug!("index created at {}", &index_path);
                        index
                    }
                    Err(create_error) => {
                        panic!(
                            "Failed to create index at {} after failing to open: {}",
                            &index_path, create_error
                        );
                    }
                }
            }
        };

        let mixed_tokenizer = MixedTokenizer::new();
        index.tokenizers().register("mixed", mixed_tokenizer);

        let index_writer = index.writer(50_000_000)?;
        let index_reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        debug!("Index writer and reader created");
        Ok(TantivyIndex {
            // schema,
            name_field,
            path_field,
            index,
            index_writer: Mutex::new(index_writer),
            index_reader,
        })
    }

    pub fn add(&self, name: &str, path: &str) -> Result<(), TantivyError> {
        let writer_guard = self.index_writer.lock().unwrap();
        writer_guard.add_document(doc!(
            self.name_field => name,
            self.path_field => path.as_bytes(),
            // self.schema.get_field("is_folder").unwrap() => is_folder,
        ))?;
        Ok(())
    }

    pub fn delete(&self, path: &str) -> Result<()> {
        let writer_guard = self.index_writer.lock().unwrap();
        writer_guard.delete_term(Term::from_field_bytes(self.path_field, path.as_bytes()));
        Ok(())
    }

    pub fn commit(&self) -> Result<(), TantivyError> {
        let mut writer_guard = self.index_writer.lock().unwrap();
        writer_guard.commit()?;
        Ok(())
    }

    pub fn search(&self, query: &str) -> Result<Vec<Something>, TantivyError> {
        debug!("Searching for {}", query);
        let mut results = vec![];
        let searcher = self.index_reader.searcher();
        let query_parser = QueryParser::for_index(&self.index, vec![self.name_field]);
        let query = query_parser.parse_query(query)?;

        let top_docs: Vec<(f32, tantivy::DocAddress)> =
            searcher.search(&query, &TopDocs::with_limit(100))?;
        debug!("Found {} results", top_docs.len());

        for (_id, (_score, doc_address)) in top_docs.iter().enumerate() {
            let retrieved_doc: TantivyDocument = searcher.doc(*doc_address)?;

            let path: String = retrieved_doc
                .get_first(self.path_field)
                .unwrap()
                .as_bytes()
                .map(|x| String::from_utf8(x.to_vec()).unwrap())
                .unwrap();

            results.push(Something {
                name: "".into(),
                path: path.into(),
                size: 0.0,
                last_modified_date: OffsetDateTime::now_utc().date(),
                class: "".into(),
            });
        }
        Ok(results)
    }

    pub fn get_num_docs(&self) -> u64 {
        self.index_reader.searcher().num_docs()
    }
}

#[derive(Clone)]
struct MixedTokenizer {
    default_tokenizer: SimpleTokenizer,
    jieba_tokenizer: JiebaTokenizer,
}

impl MixedTokenizer {
    fn new() -> Self {
        MixedTokenizer {
            default_tokenizer: SimpleTokenizer::default(),
            jieba_tokenizer: tantivy_jieba::JiebaTokenizer {},
        }
    }
}

impl Tokenizer for MixedTokenizer {
    type TokenStream<'a> = Box<dyn TokenStream + 'a>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        match detect_language(text) {
            Lang::Cmn => Box::new(self.jieba_tokenizer.token_stream(text)),
            _ => Box::new(self.default_tokenizer.token_stream(text)),
        }
    }
}

pub fn get_files(
    path: &str,
    remain_exclude_path: &Vec<String>,
) -> Result<WalkDirGeneric<((), ())>> {
    debug!("getting files from {}", path);
    let exclude_paths = remain_exclude_path.clone(); // Clone the vector to own it
    let files = WalkDir::new(path).skip_hidden(false).process_read_dir(
        move |_, _, _, dir_entry_results| {
            dir_entry_results.iter_mut().for_each(|dir_entry_result| {
                if let Ok(dir_entry) = dir_entry_result {
                    let path = dir_entry.path().to_str().unwrap().to_string();

                    if exclude_paths.iter().any(|x| path.starts_with(x)) {
                        debug!("skip path {}", path);
                        dir_entry.read_children_path = None;
                    }
                }
            })
        },
    );
    Ok(files)
}

pub fn get_subfolders(str: &str) -> Vec<String> {
    if let Ok(paths) = fs::read_dir(str) {
        return paths
            .into_iter()
            .map(|x| x.unwrap().path().to_str().unwrap().to_string())
            .collect();
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use tantivy::Document;

    use super::*;

    #[test]
    fn chinese_tokenizer() {
        let path = "./tmp";
        let _ = fs::remove_dir_all(path);
        fs::create_dir(path).ok();

        let schema_builder = Schema::builder();
        let schema = schema_builder.build();

        let index = Index::create_in_dir(&path, schema.clone()).unwrap();

        let mixed_tokenizer: MixedTokenizer = MixedTokenizer::new();
        index.tokenizers().register("mixed", mixed_tokenizer);

        {
            let mut tokenizer = index.tokenizers().get("mixed").unwrap();
            let text = "无人驾驶数据与隐私_何凡（1）.docx";
            let mut token_stream = tokenizer.token_stream(text);

            println!("建立索引分词结果:");
            token_stream.process(&mut |token| {
                println!(
                    "Token: '{}', 位置: {}, 偏移量: {}-{}",
                    token.text, token.position, token.offset_from, token.offset_to
                );
            });
        }

        {
            let mut tokenizer = index.tokenizers().get("mixed").unwrap();
            let query_text: &'static str = "中国";
            let mut token_stream = tokenizer.token_stream(query_text);

            println!("查询分词结果:");
            token_stream.process(&mut |token| {
                println!(
                    "Token: '{}', 位置: {}, 偏移量: {}-{}",
                    token.text, token.position, token.offset_from, token.offset_to
                );
            });
        }
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn chinese_doc_tokenizer() {
        let mut schema_builder = Schema::builder();
        let text_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("mixed")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        let name = schema_builder.add_text_field("name", text_options);
        let schema = schema_builder.build();

        let index = Index::create_in_ram(schema.clone());

        index.tokenizers().register("mixed", MixedTokenizer::new());
        let texts = vec![
            "中文的原神.txt",
            "你好世界.docx",
            "这是一个测试文档.pdf",
            "北京欢迎你.md",
            "今天天气怎么样.txt",
            "我喜欢学习编程.rs",
            "人工智能的未来.html",
        ];

        let mut index_writer = index.writer(50_000_000).unwrap();
        for text in texts {
            index_writer
                .add_document(doc!(
                    name => text
                ))
                .unwrap();
        }
        index_writer.commit().unwrap();

        let query_text = "北京";

        println!("\n查询分词结果:");
        let mut query_tokenizer = index.tokenizers().get("mixed").unwrap();
        let mut query_token_stream = query_tokenizer.token_stream(query_text);
        query_token_stream.process(&mut |token| {
            println!(
                "Token: '{}', 位置: {}, 偏移量: {}-{}",
                token.text, token.position, token.offset_from, token.offset_to
            );
        });

        let reader = index.reader().unwrap();
        let searcher = reader.searcher();
        let query_parser = tantivy::query::QueryParser::for_index(&index, vec![name]);
        let query = query_parser.parse_query(query_text).unwrap();
        let top_docs = searcher
            .search(&query, &tantivy::collector::TopDocs::with_limit(10))
            .unwrap();

        println!("\n查询结果:");
        for (score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address).unwrap();
            println!("文档得分: {}, 内容: {:?}", score, retrieved_doc);
            println!("{}", retrieved_doc.to_json(&schema));
        }
    }
}
