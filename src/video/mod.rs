pub mod downloader20230525;
pub mod downloader20231224;
pub mod downloader20240707;
pub mod downloader20241215;

use async_trait::async_trait;
#[allow(unused_imports)]
pub use downloader20230525::Downloader20230525;
#[allow(unused_imports)]
pub use downloader20231224::Downloader20231224;
#[allow(unused_imports)]
pub use downloader20240707::Downloader20240707;
#[allow(unused_imports)]
pub use downloader20241215::Downloader20241215;

#[allow(dead_code)]
#[async_trait]
pub trait Downloader {
    fn get_url(html: &scraper::Html) -> anyhow::Result<String>;
    async fn download_video(
        title: String,
        url: String,
        path: String,
        timeout: u64,
    ) -> anyhow::Result<()>;
}
