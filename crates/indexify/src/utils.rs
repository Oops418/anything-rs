use facade::component::anything_item::Something;
use jwalk::{WalkDir, WalkDirGeneric};
use once_cell::sync::Lazy;
use std::fs;
use std::sync::Mutex;
use tantivy::{
    Document, Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, TantivyError,
    collector::TopDocs,
    doc,
    query::{Query, QueryParser},
    schema::{
        IndexRecordOption, STORED, Schema, TEXT, TextFieldIndexing, TextOptions, Value,
        document::{ReferenceValue, ReferenceValueLeaf},
    },
    tokenizer::{NgramTokenizer, SimpleTokenizer, TokenStream, Tokenizer},
};
use tantivy_jieba::JiebaTokenizer;
use whichlang::{Lang, detect_language};

pub static TANTIVY_INDEX: Lazy<TantivyIndex> = Lazy::new(|| {
    tracing::debug!("Initializing tantivy index...");
    TantivyIndex::new().unwrap()
});

pub struct TantivyIndex {
    schema: Schema,
    index: Index,
    index_writer: Mutex<IndexWriter>,
    index_reader: IndexReader,
}

impl TantivyIndex {
    pub fn new() -> Result<Self, TantivyError> {
        tracing::debug!("Building initial tantivy index...");
        let mut schema_builder = Schema::builder();

        let name_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("mixed")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        schema_builder.add_text_field("name", name_options);
        schema_builder.add_text_field("path", STORED);
        // schema_builder.add_bytes_field("if_folder", STORED);
        let schema = schema_builder.build();

        let index_path = vaultify::get("tantivy_path").unwrap();
        let index = match Index::open_in_dir(&index_path) {
            Ok(index) => index,
            Err(_) => {
                if let Err(e) = fs::create_dir_all(&index_path) {
                    tracing::error!("Failed to create directory at {}: {}", &index_path, e);
                    panic!("Failed to create directory at {}: {}", &index_path, e);
                }
                match Index::create_in_dir(&index_path, schema.clone()) {
                    Ok(index) => index,
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

        tracing::debug!("Creating index writer and reader...");
        let index_writer = index.writer(50_000_000)?;
        let index_reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        tracing::debug!("Index writer and reader created...");
        Ok(TantivyIndex {
            schema,
            index,
            index_writer: Mutex::new(index_writer),
            index_reader,
        })
    }

    pub fn add(&self, name: &str, path: &str) -> Result<(), TantivyError> {
        let mut writer_guard = self.index_writer.lock().unwrap();
        writer_guard.add_document(doc!(
            self.schema.get_field("name").unwrap() => name,
            self.schema.get_field("path").unwrap() => path,
            // self.schema.get_field("is_folder").unwrap() => is_folder,
        ))?;
        Ok(())
    }

    pub fn search(&self, query: &str) -> Result<Vec<Something>, TantivyError> {
        tracing::debug!("Searching for {}", query);
        let mut results = vec![];
        let searcher = self.index_reader.searcher();
        let query_parser =
            QueryParser::for_index(&self.index, vec![self.schema.get_field("name").unwrap()]);
        let query = query_parser.parse_query(query)?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
        tracing::debug!("Found {} results", top_docs.len());

        for (id, (_score, doc_address)) in top_docs.iter().enumerate() {
            let retrieved_doc: TantivyDocument = searcher.doc(*doc_address)?;

            // Extract name and path from the document
            let name_field = self.schema.get_field("name").unwrap();
            let path_field = self.schema.get_field("path").unwrap();

            let name = retrieved_doc
                .get_first(name_field)
                .and_then(|field_value| {
                    if let ReferenceValue::Leaf(ReferenceValueLeaf::Str(text)) =
                        field_value.as_value()
                    {
                        Some(text)
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let path = retrieved_doc
                .get_first(path_field)
                .and_then(|field_value| {
                    if let ReferenceValue::Leaf(ReferenceValueLeaf::Str(text)) =
                        field_value.as_value()
                    {
                        Some(text)
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            // Generate random usage between 0.0 and 1.0
            let usage: f64 = id as f64 + 3.0;

            // Create Something instance with unique ID based on position in results
            results.push(Something {
                id,
                name: String::from(name).into(),
                path: String::from(path).into(),
                usage,
            });
        }
        Ok(results)
    }

    pub fn get_num_docs(&self) -> u64 {
        self.index_reader.searcher().num_docs()
    }

    pub fn commit(&self) -> Result<(), TantivyError> {
        let mut writer_guard = self.index_writer.lock().unwrap();
        writer_guard.commit()?;
        Ok(())
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
            Lang::Cmn => {
                tracing::debug!("Detected Chinese language");
                tracing::debug!("Text: {}", text);
                Box::new(self.jieba_tokenizer.token_stream(text))
            }
            _ => Box::new(self.default_tokenizer.token_stream(text)),
        }
    }
}

pub fn get_files(path: &str) -> Result<WalkDirGeneric<((), ())>, jwalk::Error> {
    tracing::debug!("Getting files from {}", path);
    let files = WalkDir::new(path).sort(false).skip_hidden(false);
    Ok(files)
}

#[cfg(test)]
mod tests {
    use jwalk::rayon::vec;

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
