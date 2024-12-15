# parse_sheet_from_weixin

The program can parse the music sheet from weixin.

## Usage

* Install chromedriver
  * You can follow the tutorial here: [Run Selenium and Chrome on WSL2 using Python and Selenium webdriver](https://cloudbytes.dev/snippets/run-selenium-and-chrome-on-wsl2)
  * Here is [the latest chromedriver](https://googlechromelabs.github.io/chrome-for-testing/#stable)
* Create `urls.txt` and put urls into it
* Create `qq_urls.txt` and put the QQ link inside
  * You need to find the corresponding link [here](https://v.qq.com/biu/creator/home?vcuid=9000001247)
  * The bilibili link [here](https://space.bilibili.com/388464704)
* Run

```shell
# Terminal 1: Run chromedriver
chromedriver --port=9515
# Terminal 2: Run this program
RUST_LOG=info cargo run
```
