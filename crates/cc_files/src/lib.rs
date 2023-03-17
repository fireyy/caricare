mod syntax_highlighting;

use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    io::Write,
    time::{Duration, Instant},
};

use anyhow::Context as _;
use egui::Vec2;
use egui_extras::RetainedImage;
use image::ImageFormat;
use tokio_stream::StreamExt as _;

pub enum FileType {
    StaticImage(RetainedImage),
    AnimatedImage(Animated),
    PlainText(Vec<u8>),
    Unknown,
}

impl std::fmt::Debug for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StaticImage(img) => f
                .debug_struct("StaticImage")
                .field("name", &img.debug_name())
                .field("size", &img.size())
                .finish(),
            Self::AnimatedImage(img) => f
                .debug_struct("AnimatedImage")
                .field("frames", &img.frames.len())
                .field("intervals", &img.intervals.len())
                .finish(),
            Self::PlainText(data) => f
                .debug_struct("PlainText")
                .field("size", &data.len())
                .finish(),
            _ => f.debug_struct("Unknown File").finish(),
        }
    }
}
impl FileType {
    pub fn is_image(&self) -> bool {
        matches!(self, Self::StaticImage(_) | Self::AnimatedImage(_))
    }

    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            Self::PlainText(data) => {
                if let Ok(text) = std::str::from_utf8(data) {
                    syntax_highlighting::code_view_ui(ui, text)
                } else {
                    ui.label("Parse error")
                }
            }
            _ => ui.centered_and_justified(|ui| ui.label("Unknown")).inner,
        }
    }

    pub fn show_size(&self, ui: &mut egui::Ui, size: Vec2) -> egui::Response {
        match self {
            Self::StaticImage(image) => ui.add(
                egui::Image::new(image.texture_id(ui.ctx()), size)
                    .sense(egui::Sense::hover().union(egui::Sense::click())),
            ),
            Self::AnimatedImage(image) => {
                let dt = ui.input(|i| i.stable_dt.min(0.1));

                if let Some((img, delay)) = image.frame(dt) {
                    let resp = ui.add(
                        egui::Image::new(img.texture_id(ui.ctx()), size)
                            .sense(egui::Sense::hover().union(egui::Sense::click())),
                    );

                    ui.ctx().request_repaint_after(delay);
                    return resp;
                }
                ui.allocate_response(size, egui::Sense::hover().union(egui::Sense::click()))
            }
            Self::PlainText(data) => {
                if let Ok(text) = std::str::from_utf8(data) {
                    syntax_highlighting::code_view_ui(ui, text)
                } else {
                    ui.label("Parse error")
                }
            }
            _ => ui.label("Unknown"),
        }
    }

    pub fn load_retained_image(name: &str, data: &[u8]) -> anyhow::Result<RetainedImage> {
        RetainedImage::from_image_bytes(name, data)
            .map_err(|err| anyhow::anyhow!("cannot load '{name}': {err}"))
    }

    fn load_svg(name: &str, data: &[u8]) -> anyhow::Result<RetainedImage> {
        RetainedImage::from_svg_bytes(name, data)
            .map_err(|err| anyhow::anyhow!("cannot load '{name}': {err}"))
    }

    pub fn size_vec2(&self) -> egui::Vec2 {
        match self {
            FileType::StaticImage(img) => img.size_vec2(),
            FileType::AnimatedImage(img) => img.size_vec2(),
            _ => egui::Vec2::default(),
        }
    }
}

pub struct Animated {
    frames: Vec<RetainedImage>,
    intervals: Vec<Duration>,
    position: Cell<usize>,
    last: Cell<Option<Instant>>,
}

impl Animated {
    pub fn frame(&self, dt: f32) -> Option<(&RetainedImage, Duration)> {
        let pos = self.position.get();
        let delay = self.intervals.get(pos)?;

        match self.last.get() {
            Some(last) if last.elapsed().as_secs_f32() >= delay.as_secs_f32() - dt => {
                self.position.set((pos + 1) % self.frames.len());
                self.last.set(Some(Instant::now()));
            }
            Some(..) => {}
            None => {
                self.last.set(Some(Instant::now()));
            }
        }

        self.frames.get(pos).map(|frame| (frame, *delay))
    }

    pub fn load_apng(name: &str, data: &[u8]) -> anyhow::Result<Self> {
        use image::ImageDecoder as _;
        let dec = image::codecs::png::PngDecoder::new(data)?;
        anyhow::ensure!(dec.is_apng(), "expected an animated png");
        Self::load_frames(name, dec.total_bytes() as _, dec.apng())
    }

    pub fn load_gif(name: &str, data: &[u8]) -> anyhow::Result<Self> {
        use image::ImageDecoder as _;
        let dec = image::codecs::gif::GifDecoder::new(data)?;
        Self::load_frames(name, dec.total_bytes() as _, dec)
    }

    pub fn load_webp(name: &str, data: &[u8]) -> anyhow::Result<Self> {
        use image::ImageDecoder as _;
        let dec = image::codecs::webp::WebPDecoder::new(data)?;
        Self::load_frames(name, dec.total_bytes() as _, dec)
    }

    pub fn size_vec2(&self) -> egui::Vec2 {
        if let Some(img) = self.frames.get(0) {
            img.size_vec2()
        } else {
            egui::Vec2::default()
        }
    }

    fn load_frames<'a>(
        name: &str,
        hint: usize,
        decoder: impl image::AnimationDecoder<'a>,
    ) -> anyhow::Result<Self> {
        let mut buf = std::io::Cursor::new(Vec::with_capacity(hint));

        let (mut frames, mut intervals) = (vec![], vec![]);

        for (i, frame) in decoder.into_frames().enumerate() {
            let frame = frame?;
            let delay = Duration::from(frame.delay());

            // TODO use DynamicImage instead
            let buffer = frame.into_buffer();
            buffer.write_to(&mut buf, ImageFormat::Png)?; // we need to change the alpha here maybe
            buf.flush().expect("flush image during transcode");

            let pos = buf.position();
            buf.set_position(0);

            let image = FileType::load_retained_image(
                &format!("{name}_{i}"),
                &buf.get_ref()[..pos as usize],
            )
            .with_context(|| anyhow::anyhow!("cannot decode frame: {i}"))?;
            frames.push(image);
            intervals.push(delay);
        }

        Ok(Self {
            frames,
            intervals,
            position: Cell::default(),
            last: Cell::default(),
        })
    }
}

pub struct Cache {
    map: HashMap<String, FileType>,
    loader: Loader,
}

impl Cache {
    pub fn create(repaint: impl Fn() + Clone + Send + Sync + 'static) -> Self {
        Self {
            map: HashMap::default(),
            loader: Loader::spawn(repaint),
        }
    }

    pub fn get(&mut self, url: &str) -> Option<&FileType> {
        match self.map.get(url) {
            Some(img) => Some(img),
            None => {
                self.loader.request(url);
                None
            }
        }
    }

    pub fn check(&mut self, url: &str) -> Option<&FileType> {
        self.map.get(url)
    }

    pub fn poll(&mut self) {
        for (k, v) in self.loader.produce.try_iter() {
            self.map.insert(k, v);
        }
    }

    pub fn add(&mut self, name: &str, data: Vec<u8>) {
        tracing::debug!("Add image: {name}");
        self.map.insert(name.to_string(), FileType::Unknown);
        if let Ok(file) = Loader::load(name, data) {
            self.map.insert(name.to_string(), file);
        }
    }

    pub fn replace(&mut self, old_key: &str, new_key: &str) {
        if let Some(v) = self.map.remove(old_key) {
            self.map.insert(new_key.to_string(), v);
        }
    }
}

#[derive(Clone)]
struct Loader {
    submit: flume::Sender<String>,
    produce: flume::Receiver<(String, FileType)>,
}

impl Loader {
    fn spawn(repaint: impl Fn() + Clone + Send + Sync + 'static) -> Self {
        let (submit, submit_rx) = flume::unbounded::<String>();
        let (produce_tx, produce) = flume::unbounded();

        cc_runtime::spawn(async move {
            let mut seen = HashSet::new();
            let mut stream = submit_rx.into_stream();
            let client = reqwest::Client::new();

            while let Some(url) = stream.next().await {
                if !seen.insert(url.clone()) {
                    continue;
                }

                let client = client.clone();
                let tx = produce_tx.clone();
                let repaint = repaint.clone();

                tokio::spawn(async move {
                    let Some(data) = Self::fetch(client, &url).await else { return };

                    tokio::task::spawn_blocking(move || match Self::load(&url, data) {
                        Ok(img) => {
                            let _ = tx.send((url, img));
                            repaint();
                        }
                        Err(err) => eprintln!("cannot fetch: {url}: {err}"),
                    });
                });
            }
        });

        Self { submit, produce }
    }

    fn request(&self, url: &str) {
        let _ = self.submit.send(url.to_string());
    }

    // TODO cache this
    async fn fetch(client: reqwest::Client, url: &str) -> Option<Vec<u8>> {
        eprintln!("getting: {url}");
        let resp = client.get(url).send().await.ok()?;
        if resp.status().as_u16() == 404 {
            // TODO report this
            eprintln!("cannot fetch: {url}: 404 not found");
            return None;
        }

        resp.bytes().await.ok().map(|d| d.to_vec())
    }

    fn load(name: &str, data: Vec<u8>) -> anyhow::Result<FileType> {
        if let Some(format) = infer::get(&data[..data.len().min(128)]) {
            tracing::debug!("Load type: {:?}", format);
            let file = match format.mime_type() {
                "image/png" => {
                    let dec = image::codecs::png::PngDecoder::new(&*data).map_err(|err| {
                        anyhow::anyhow!("expected png, got something else for '{name}': {err}")
                    })?;

                    if dec.is_apng() {
                        Animated::load_apng(name, &data).map(FileType::AnimatedImage)?
                    } else {
                        FileType::load_retained_image(name, &data).map(FileType::StaticImage)?
                    }
                }
                "image/jpeg" => {
                    FileType::load_retained_image(name, &data).map(FileType::StaticImage)?
                }
                "image/gif" => Animated::load_gif(name, &data).map(FileType::AnimatedImage)?,
                // FIXME webp decode bug: https://github.com/image-rs/image/issues/1856
                "image/webp" => match data.get(44..48).filter(|bytes| bytes == b"ANMF") {
                    Some(_) => {
                        tracing::debug!("animated webp");
                        Animated::load_webp(name, &data).map(FileType::AnimatedImage)?
                    }
                    None => {
                        FileType::load_retained_image(name, &data).map(FileType::StaticImage)?
                    }
                },
                "image/svg+xml" => FileType::load_svg(name, &data).map(FileType::StaticImage)?,
                _ => FileType::PlainText(data),
            };
            Ok(file)
        } else {
            Ok(FileType::PlainText(data))
        }
    }
}
