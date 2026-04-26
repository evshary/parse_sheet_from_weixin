# parse_sheet_from_weixin

The program can parse the music sheet from weixin.

## Usage

* Run the geckodriver inside snap: `/snap/bin/geckodriver`
* Create `urls.txt` and put urls into it
* Create `bilibili_urls.txt` and put [the corressponding bilibili urls](https://space.bilibili.com/388464704/upload/video) into it
* Run

```shell
# Terminal 1: Run geckodriver
/snap/bin/geckodriver
# Terminal 2: Run this program
RUST_LOG=info cargo run
```

## For developers

Remember to run `pre-commit install --install-hooks` to ensure every commit follows the rules.
