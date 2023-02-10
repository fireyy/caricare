# Caricare

<p align="center">
    <img src="res/screenshot_main.png" alt="main screen" width="600">
</p>

### Develop

Copy `.env.sample` to `.env` and set

```
ALIYUN_KEY_ID=""
ALIYUN_KEY_SECRET=""
ALIYUN_ENDPOINT=""
ALIYUN_BUCKET=""
ALIYUN_BUCKET_PATH=""
CDN_URL=""
```

`ALIYUN_BUCKET_PATH` and `CDN_URL` is optional.

Run dev

```
cargo run -p cc-gui
```

### Build

```
cargo build -p cc-gui
```