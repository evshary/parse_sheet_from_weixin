use log::info;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use std::{
    fs::{self, File},
    io::Write,
};

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
        let selector = Selector::parse("mpvoice").unwrap();
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
    async fn download(self) {
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
            let resp = reqwest::get(self.accompaniment)
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
            // Get the video title
            let resp = reqwest::get(&self.video).await.expect("Request failed");
            let html = resp.text().await.expect("Invalid body");
            let document = Html::parse_document(&html);
            let selector = Selector::parse("meta").unwrap();
            let title = document
                .select(&selector)
                .filter(|x| x.value().attr("name").unwrap_or_default() == "description")
                .collect::<Vec<ElementRef>>()[0]
                .value()
                .attr("content")
                .unwrap();
            // Download video: Send request to http://www.weibodang.cn/videoextractor/extract.php
            let parse_url = String::from("http://www.weibodang.cn/videoextractor/extract.php");
            let params = [
                (
                    "csrfmiddlewaretoken",
                    "2D7YysIrxAkqAlucG5CBY6Kou7pSWP7WpucCPG2SiH4mL1lVezNVG2nSSZYNRDmx",
                ),
                ("q", &self.video),
                (
                    "check",
                    "%C2%A0%C2%A0%C2%A0Find+Video+Link%C2%A0%C2%A0%C2%A0",
                ),
            ];
            let cookie = String::from("lang=eng; csrftoken=pEv2hlb0riSLHGFvfhNUHodIToTi9rXcMvAGyzvrcpCHSmweNLYepkQchgsd4fcN; Hm_lvt_9918e92916590d12525d5fc1be3d1d5f=1675578695; Hm_lpvt_9918e92916590d12525d5fc1be3d1d5f=1675585799");
            let client = reqwest::Client::new();
            let resp = client
                .post(parse_url)
                .header(reqwest::header::COOKIE, cookie)
                .form(&params)
                .send()
                .await
                .expect("Request failed");
            let html = resp.text().await.expect("Invalid body");
            let document = Html::parse_document(&html);
            let selector = Selector::parse("video").unwrap();
            let video_url = document
                .select(&selector)
                .nth(0) // Get first element
                .unwrap()
                .value()
                .attr("src")
                .unwrap();
            let resp = reqwest::get(video_url).await.expect("Request failed");
            let binary = resp.bytes().await.expect("Invalid body");
            let mut file =
                File::create(format!("{}/{}.mp4", path, title)).expect("Failed to create video");
            file.write_all(&binary).expect("Failed to create video");
        }

        // Download sheet
        {
            info!("Dowloading sheets...");
            for (idx, sheet) in self.sheets.into_iter().enumerate() {
                let resp = reqwest::get(sheet).await.expect("Request failed");
                let binary = resp.bytes().await.expect("Invalid body");
                let mut file = File::create(format!("{}/{}.png", path, idx + 1))
                    .expect("Failed to create sheet");
                file.write_all(&binary).expect("Failed to create 伴奏");
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
