use scraper::{Html, Selector};
use std::thread;
use std::time::Duration;
use std::{
    fs::{self, File},
    io::Write,
};
use thirtyfour::prelude::*;

const OUTPUT: &str = "output";

struct Sheet {
    url: String,
    title: String,
    accompaniment: String,
    video: String,
    sheets: Vec<String>,
}

impl Sheet {
    async fn try_new(url: String) -> Result<Sheet, Box<dyn std::error::Error>> {
        // Get Html
        log::info!("The URL: {}", url);
        let resp = reqwest::get(url.clone()).await?;
        let html = resp.text().await?;
        let document = Html::parse_document(&html);

        // Get the title
        // Get the inner_html under h1
        let selector = Selector::parse("h1")?;
        let mut title = document
            .select(&selector)
            .nth(0) // Get first element
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unable to get title",
            ))?
            .inner_html();
        title.retain(|c| !"\t\r\n".contains(c));
        let splits = title.trim().split('|').collect::<Vec<&str>>();
        let title = String::from(splits[1]) + " - " + splits[0];
        log::info!("Parsed title: {}", title);

        // Get the accompaniment
        // Get the attr voice_encode_fileid of mpvoice
        let selector = Selector::parse("mp-common-mpaudio")?;
        let voice_id = document
            .select(&selector)
            .nth(0)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unable to get accompaniment url",
            ))?
            .value()
            .attr("voice_encode_fileid")
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unable to get accompaniment url",
            ))?;
        let accompaniment = format!("https://res.wx.qq.com/voice/getvoice?mediaid={}", voice_id);
        log::info!("Parsed voice URL: {}", accompaniment);

        // Get the url of video
        // Get the attr data-src of iframe
        let selector = Selector::parse("iframe")?;
        let qq_url = document
            .select(&selector)
            .nth(0)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unable to get video url",
            ))?
            .value()
            .attr("data-src")
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unable to get video url",
            ))?;
        let re = regex::Regex::new(r"vid=([[:alnum:]]+)")?;
        let vid = re.captures(qq_url).ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Unable to get video url",
        ))?;
        let video = format!("https://v.qq.com/x/page/{}.html", &vid[1]);
        log::info!("Parsed QQ video URL: {}", video);

        // Get the music sheet
        // Get the attr data-src of img with class js_insertlocalimg
        let selector = Selector::parse("img")?;
        let imgs = document.select(&selector).filter(|x| {
            x.value()
                .attr("class")
                .unwrap_or_default()
                .contains("js_insertlocalimg")
        });
        let sheets = imgs
            .map(|img| match img.value().attr("data-src") {
                Some(src) => src.to_string(),
                None => {
                    log::warn!("Unabel to get sheet url");
                    "".to_owned()
                }
            })
            .collect::<Vec<String>>();
        log::info!("Parsed sheet URL: {:?}", sheets);
        Ok(Sheet {
            url,
            title,
            accompaniment,
            video,
            sheets,
        })
    }

    async fn download_video(
        &self,
        url: &str,
        path: &str,
        timeout: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Send request via selenium
        let caps = DesiredCapabilities::chrome();
        let driver = WebDriver::new("http://localhost:9515", caps).await?;
        //driver
        //    .set_implicit_wait_timeout(Duration::new(timeout, 0))
        //    .await?;
        //let timeouts = driver.get_timeouts().await?;
        match driver.goto(url).await {
            Ok(_) => {}
            Err(e) => {
                log::info!("{:?}", e);
                log::info!("You can ignore this meesage.")
            }
        }
        // Waiting for selenium
        thread::sleep(Duration::new(timeout, 0));
        let html = driver.source().await?;
        let document = Html::parse_document(&html);
        // Get the video title
        let selector = Selector::parse("title")?;
        let title = document
            .select(&selector)
            .nth(0) // Get first element
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unable to get video title",
            ))?
            .inner_html();
        log::info!("Downloaded video title: {}", title);
        // Get video url
        let selector = Selector::parse("video")?;
        let video_url = document
            .select(&selector)
            .nth(0) // Get first element
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unable to get video url",
            ))?
            .value()
            .attr("src")
            .unwrap();
        log::info!("Downloaded video url: {}", video_url);
        // Download video as a file
        let resp = reqwest::get(video_url).await?;
        let binary = resp.bytes().await?;
        let mut file = File::create(format!("{}/{}.mp4", path, title))?;
        file.write_all(&binary)?;
        driver.quit().await?;
        Ok(())
    }

    async fn download(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create folder
        log::info!("Creating folder...");
        let path = format!("{}/{}", OUTPUT, self.title.as_str());
        fs::create_dir_all(&path)?;

        // Create url.txt
        {
            log::info!("Creating url.txt...");
            let mut file = File::create(format!("{}/url.txt", path))?;
            file.write_all(self.url.as_bytes())?;
        }

        // Download accompaniment
        {
            log::info!("Dowloading accompaniment...");
            let resp = reqwest::get(self.accompaniment.clone()).await?;
            let binary = resp.bytes().await?;
            let mut file = File::create(format!("{}/伴奏.mp3", path))?;
            file.write_all(&binary)?;
        }

        // Download video
        {
            log::info!("Dowloading video...");
            // Timeout means we need to wait for ad play and load the video we want
            self.download_video(&self.video, &path, 30).await?;
        }

        // Download sheet
        {
            log::info!("Dowloading sheets...");
            for (idx, sheet) in self.sheets.clone().into_iter().enumerate() {
                let resp = reqwest::get(sheet).await?;
                let binary = resp.bytes().await?;
                let mut file = File::create(format!("{}/{}.png", path, idx + 1))?;
                file.write_all(&binary)?;
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let file_content = fs::read_to_string("urls.txt")?;
    let urls = file_content.split('\n');
    for url in urls {
        // Don't access the website too fast
        thread::sleep(Duration::new(5, 0));
        // Parse the resource
        let sheet = match Sheet::try_new(url.to_string()).await {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Failed to parse sheet: {:?}", e);
                continue;
            }
        };
        // Download the resource
        match sheet.download().await {
            Ok(_) => {}
            Err(e) => {
                log::warn!("Failed to download sheet: {:?}", e);
                continue;
            }
        }
        // Add newline to separate
        log::info!("-----------------------------------------------------------------------------------------------");
    }
    Ok(())
}
