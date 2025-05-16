use jwalk::{WalkDir, WalkDirGeneric};
use once_cell::sync::Lazy;
use std::fs;
use std::sync::Mutex;
use tantivy::{
    Document, Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, TantivyError,
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{STORED, Schema, TEXT},
};

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
        schema_builder.add_text_field("name", TEXT | STORED);
        schema_builder.add_text_field("path", STORED);
        // schema_builder.add_bytes_field("if_folder", STORED);
        let schema = schema_builder.build();

        let index_path = vaultify::get("tantivy_path").unwrap();
        let index = match Index::open_in_dir(&index_path) {
            Ok(index) => index,
            Err(_) => {
                if let Err(e) = fs::create_dir_all(&index_path) {
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

    pub fn search(&self, query: &str) -> Result<(), TantivyError> {
        tracing::debug!("Searching for {}", query);
        let searcher = self.index_reader.searcher();
        let query_parser =
            QueryParser::for_index(&self.index, vec![self.schema.get_field("name").unwrap()]);
        let query = query_parser.parse_query(query)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
        tracing::debug!("Found {} results", top_docs.len());
        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
            println!("{}", retrieved_doc.to_json(&self.schema));
        }
        Ok(())
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

pub fn get_files(path: &str) -> Result<WalkDirGeneric<((), ())>, jwalk::Error> {
    tracing::debug!("Getting files from {}", path);
    let files = WalkDir::new(path).sort(false).skip_hidden(false);
    Ok(files)
}
