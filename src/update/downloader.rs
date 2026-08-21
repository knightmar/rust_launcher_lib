use crate::update::errors::UpdateErrors;
use crate::update::structs::UpdateFile;
use crate::update::verify_file;
use futures_util::future::join_all;
use futures_util::StreamExt;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;

pub struct Downloader {
    pub files: Vec<UpdateFile>,
    pub max_concurrent_downloads: usize,
    failed_files: Vec<UpdateFile>,
}

impl Downloader {
    pub fn new(max_concurrent_downloads: usize) -> Self {
        Downloader {
            files: vec![],
            max_concurrent_downloads,
            failed_files: vec![],
        }
    }

    pub fn with_files(mut self, files: Vec<UpdateFile>) -> Self {
        self.files = files;
        self
    }

    fn client() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap()
    }

    pub async fn download_single_file(
        file: UpdateFile,
        client: reqwest::Client,
    ) -> Result<(), UpdateErrors> {
        let file1 = file.clone();
        let path = Path::new(file1.local_path.as_str());

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| UpdateErrors::Install(format!("Failed to create directory: {}", e)))?;
        }

        if path.exists() {
            println!(
                "  - File: {} already found, checking if valid...",
                file.local_path
            );
            match verify_file(file.clone()).await {
                Ok(true) => return Ok(()),
                Ok(false) => {
                    tokio::fs::remove_file(&path).await.map_err(|e| {
                        UpdateErrors::Install(format!("Failed to remove file: {}", e))
                    })?;
                }
                Err(e) => return Err(e),
            }
        }

        println!("  - File: {} not found, downloading", file.local_path);

        let temp_path = path.with_extension("tmp");
        let mut temp_file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(|e| UpdateErrors::Install(format!("Failed to create temp file: {}", e)))?;

        let mut stream = client
            .get(file.url.clone())
            .send()
            .await
            .map_err(|e1| UpdateErrors::Download(e1.to_string()))?
            .bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| UpdateErrors::Download(e.to_string()))?;
            temp_file
                .write_all(&chunk)
                .await
                .map_err(|e| UpdateErrors::Install(format!("Failed to write chunk: {}", e)))?;
        }

        temp_file
            .flush()
            .await
            .map_err(|e| UpdateErrors::Install(e.to_string()))?;

        temp_file
            .sync_all()
            .await
            .map_err(|e| UpdateErrors::Install(format!("Failed to sync temp file: {}", e)))?;

        tokio::fs::rename(&temp_path, path)
            .await
            .map_err(|e| UpdateErrors::Install(format!("Failed to rename file: {}", e)))?;
        Ok(())
    }

    pub async fn download_all_files(&mut self) -> Result<Vec<UpdateFile>, UpdateErrors> {
        let mut failed_files = Vec::new();

        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_downloads));
        let client = Self::client();

        let tasks = self.files.iter().map(|file| {
            let file = file.clone();
            let semaphore = semaphore.clone();
            let client = client.clone();

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                Self::download_single_file(file, client).await
            })
        });

        let results = join_all(tasks).await;
        for (file, result) in self.files.iter().zip(results) {
            match result {
                Ok(Ok(())) => continue,
                Ok(Err(e)) => {
                    eprintln!("Download failed for {}: {:?}", file.local_path, e);
                    failed_files.push(file.clone());
                }
                Err(e) => {
                    eprintln!("Task panicked for {}: {}", file.local_path, e);
                    failed_files.push(file.clone());
                }
            }
        }

        self.failed_files = failed_files;
        Ok(self.failed_files.clone())
    }

    pub fn failed_files(&self) -> &Vec<UpdateFile> {
        &self.failed_files
    }
}
