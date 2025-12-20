use std::{process::Command, sync::Mutex};

use async_trait::async_trait;

use crate::video::Downloader;

#[allow(dead_code)]
const URLS_FILE: &str = "bilibili_urls.txt";

#[allow(dead_code)]
static VIDEO_IDX: Mutex<usize> = Mutex::new(0);

#[allow(dead_code)]
pub struct Downloader20251220;

impl Downloader20251220 {
    pub fn get_url_by_index(index: usize) -> anyhow::Result<String> {
        let file_content = std::fs::read_to_string(URLS_FILE)?;
        let urls = file_content.split('\n').collect::<Vec<_>>();
        urls.get(index)
            .map(std::string::ToString::to_string)
            .ok_or_else(|| anyhow::anyhow!("Index out of bound"))
    }
}

#[async_trait]
impl Downloader for Downloader20251220 {
    fn get_url(_document: &scraper::Html) -> anyhow::Result<String> {
        Err(anyhow::anyhow!("Use get_url_by_index instead"))
    }

    async fn download_video(
        _title: String,
        url: String,
        path: String,
        _timeout: u64,
    ) -> anyhow::Result<()> {
        let output = format!("{path}");
        let status = Command::new("yt-dlp")
            .arg("-f")
            .arg("bestvideo+bestaudio")
            .arg("-P")
            .arg(output)
            .arg(url)
            .status()?;
        if !status.success() {
            return Err(anyhow::anyhow!("yt-dlp failed"));
        }
        Ok(())
    }
}
