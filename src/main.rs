use log::info;
use regex::Regex;
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
    fn new(url: String, html: String) -> Sheet {
        info!("Parse URL: {}", url);
        let document = Html::parse_document(&html);
        // Get the title
        // Get the inner_html under h1
        let selector = Selector::parse("h1").unwrap();
        let mut title = document
            .select(&selector)
            .nth(0) // Get first element
            .unwrap()
            .inner_html();
        title.retain(|c| !"\t\r\n".contains(c));
        let splits = title.trim().split('|').collect::<Vec<&str>>();
        let title = String::from(splits[1]) + " - " + splits[0];
        info!("Title: {}", title);

        // Get the accompaniment
        // Get the attr voice_encode_fileid of mpvoice
        let selector = Selector::parse("mp-common-mpaudio").unwrap();
        let voice_id = document
            .select(&selector)
            .nth(0)
            .unwrap()
            .value()
            .attr("voice_encode_fileid")
            .unwrap();
        let accompaniment = format!("https://res.wx.qq.com/voice/getvoice?mediaid={}", voice_id);
        info!("Voice URL: {}", accompaniment);

        // Get the url of video
        // Get the attr data-src of iframe
        let selector = Selector::parse("iframe").unwrap();
        let qq_url = document
            .select(&selector)
            .nth(0)
            .unwrap()
            .value()
            .attr("data-src")
            .unwrap();
        let re = Regex::new(r"vid=([[:alnum:]]+)").unwrap();
        let vid = re.captures(qq_url).unwrap();
        let video = format!("https://v.qq.com/x/page/{}.html", &vid[1]);
        info!("Video URL: {}", video);

        // Get the music sheet
        // Get the attr data-src of img with class js_insertlocalimg
        let selector = Selector::parse("img").unwrap();
        let imgs = document.select(&selector).filter(|x| {
            x.value()
                .attr("class")
                .unwrap_or_default()
                .contains("js_insertlocalimg")
        });
        let sheets = imgs
            .map(|img| img.value().attr("data-src").unwrap().to_string())
            .collect::<Vec<String>>();
        info!("{:?}", sheets);
        Sheet {
            url,
            title,
            accompaniment,
            video,
            sheets,
        }
    }

    async fn download_video(&self, url: &str, path: &str, timeout: u64) -> WebDriverResult<()> {
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
                info!("{:?}", e);
                info!("You can ignore this meesage.")
            }
        }
        // Waiting for selenium
        thread::sleep(Duration::new(timeout, 0));
        let html = driver.source().await?;
        let document = Html::parse_document(&html);
        // Get the video title
        let selector = Selector::parse("title").unwrap();
        let title = document
            .select(&selector)
            .nth(0) // Get first element
            .unwrap()
            .inner_html();
        // Get video url
        let selector = Selector::parse("video").unwrap();
        let video_url = document
            .select(&selector)
            .nth(0) // Get first element
            .unwrap()
            .value()
            .attr("src")
            .unwrap();
        // Download video as a file
        let resp = reqwest::get(video_url).await.expect("Request failed");
        let binary = resp.bytes().await.expect("Invalid body");
        let mut file =
            File::create(format!("{}/{}.mp4", path, title)).expect("Failed to create video");
        file.write_all(&binary).expect("Failed to create video");
        driver.quit().await?;
        Ok(())
    }

    async fn download(&self) {
        // Create folder
        info!("Creating folder...");
        let path = format!("{}/{}", OUTPUT, self.title.as_str());
        fs::create_dir_all(&path).unwrap();

        // Create url.txt
        {
            info!("Creating url.txt...");
            let mut file =
                File::create(format!("{}/url.txt", path)).expect("Failed to create file");
            file.write_all(self.url.as_bytes())
                .expect("Failed to create url.txt");
        }

        // Download accompaniment
        {
            info!("Dowloading accompaniment...");
            let resp = reqwest::get(self.accompaniment.clone())
                .await
                .expect("Request failed");
            let binary = resp.bytes().await.expect("Invalid body");
            let mut file =
                File::create(format!("{}/伴奏.mp3", path)).expect("Failed to create 伴奏");
            file.write_all(&binary).expect("Failed to create 伴奏");
        }

        // Download video
        {
            info!("Dowloading video...");
            self.download_video(&self.video, &path, 30)
                .await
                .expect("selenium failed");
        }

        // Download sheet
        {
            info!("Dowloading sheets...");
            for (idx, sheet) in self.sheets.clone().into_iter().enumerate() {
                let resp = reqwest::get(sheet).await.expect("Request failed");
                let binary = resp.bytes().await.expect("Invalid body");
                let mut file = File::create(format!("{}/{}.png", path, idx + 1))
                    .expect("Failed to create sheet");
                file.write_all(&binary).expect("Failed to create sheets");
            }
        }

        // Add newline
        info!("-----------------------------------------------------------------------------------------------");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let file_content = fs::read_to_string("urls.txt")?;
    let urls = file_content.split('\n');
    for url in urls {
        let resp = reqwest::get(url).await?;
        let html = resp.text().await?;
        let sheet = Sheet::new(url.to_string(), html);
        sheet.download().await;
    }
    Ok(())
}
