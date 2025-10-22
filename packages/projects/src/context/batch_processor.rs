use tokio::sync::mpsc;
use tokio::task::JoinSet;
use std::path::PathBuf;
use super::incremental_parser::{IncrementalParser, ParsedFile};

/// Processes multiple files concurrently with controlled parallelism
pub struct BatchProcessor {
    max_concurrent: usize,
}

impl BatchProcessor {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent: max_concurrent.max(1), // At least 1
        }
    }

    /// Process all files in a directory with concurrent parsing
    pub async fn process_directory(&self, dir: PathBuf, file_extensions: &[&str]) -> Vec<ProcessedFileResult> {
        let files = self.collect_files(&dir, file_extensions);
        self.process_files(files).await
    }

    /// Process a list of specific files
    pub async fn process_files(&self, files: Vec<PathBuf>) -> Vec<ProcessedFileResult> {
        let (tx, mut rx) = mpsc::channel(100);
        let mut tasks = JoinSet::new();
        
        let total_files = files.len();
        tracing::info!("Processing {} files with max {} concurrent tasks", total_files, self.max_concurrent);

        // Process files in batches
        for chunk in files.chunks(self.max_concurrent) {
            for path in chunk {
                let tx_clone = tx.clone();
                let path_clone = path.clone();

                tasks.spawn(async move {
                    let result = process_single_file(path_clone).await;
                    let _ = tx_clone.send(result).await;
                });
            }

            // Wait for current batch to complete before starting next
            while tasks.len() >= self.max_concurrent {
                if tasks.join_next().await.is_none() {
                    break;
                }
            }
        }

        drop(tx);

        // Wait for remaining tasks
        while tasks.join_next().await.is_some() {}

        // Collect all results
        let mut results = Vec::new();
        while let Some(result) = rx.recv().await {
            results.push(result);
        }

        tracing::info!("Completed processing {} files", results.len());
        results
    }

    fn collect_files(&self, dir: &PathBuf, file_extensions: &[&str]) -> Vec<PathBuf> {
        walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                let path = e.path();
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    file_extensions.contains(&ext)
                } else {
                    false
                }
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    }
}

async fn process_single_file(path: PathBuf) -> ProcessedFileResult {
    let mut parser = IncrementalParser::new();
    
    match parser.parse_file(&path) {
        Ok(parsed) => {
            let tokens = estimate_tokens(&parsed);
            ProcessedFileResult {
                path,
                success: true,
                symbols: parsed.symbols.iter().map(|s| s.name.clone()).collect(),
                dependencies: parsed.dependencies,
                tokens,
                error: None,
            }
        }
        Err(e) => {
            ProcessedFileResult {
                path,
                success: false,
                symbols: vec![],
                dependencies: vec![],
                tokens: 0,
                error: Some(e),
            }
        }
    }
}

pub struct ProcessedFileResult {
    pub path: PathBuf,
    pub success: bool,
    pub symbols: Vec<String>,
    pub dependencies: Vec<String>,
    pub tokens: usize,
    pub error: Option<String>,
}

fn estimate_tokens(parsed: &ParsedFile) -> usize {
    // Rough estimation: each symbol ~50 tokens, plus dependencies
    let symbol_tokens = parsed.symbols.len() * 50;
    let dep_tokens = parsed.dependencies.len() * 20;
    symbol_tokens + dep_tokens
}

impl Default for BatchProcessor {
    fn default() -> Self {
        // Default to number of CPUs or 4, whichever is smaller
        let max_concurrent = num_cpus::get().min(4);
        Self::new(max_concurrent)
    }
}
