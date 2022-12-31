use log::info;
use regex::Regex;
use scraper::{Html, Selector};
use std::{
    fs::{self, File},
    io::Write,
};

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
    fn download(self) {
        // Create folder
        fs::create_dir(self.title.as_str()).unwrap();
        // Create url.txt
        let mut file = File::create(self.title.clone() + "/url.txt").unwrap();
        file.write_all(self.url.as_bytes()).unwrap();
        // Download accompaniment
        // Download video
        // Download sheet
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
        sheet.download();
    }
    Ok(())
}
