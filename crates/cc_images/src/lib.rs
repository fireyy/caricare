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

pub enum Image {
    Static(RetainedImage),
    Animated(Animated),
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Static(img) => f
                .debug_struct("StaticImage")
                .field("name", &img.debug_name())
                .field("size", &img.size())
                .finish(),
            Self::Animated(img) => f
                .debug_struct("AnimatedImage")
                .field("frames", &img.frames.len())
                .field("intervals", &img.intervals.len())
                .finish(),
        }
    }
}
impl Image {
    pub fn show_size(&self, ui: &mut egui::Ui, size: Vec2) -> egui::Response {
        match self {
            Self::Static(image) => ui.add(
                egui::Image::new(image.texture_id(ui.ctx()), size)
                    .sense(egui::Sense::hover().union(egui::Sense::click())),
            ),
            Self::Animated(image) => {
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
            Image::Static(img) => img.size_vec2(),
            Image::Animated(_img) => egui::Vec2::new(1.0, 1.0), //TODO: Animated Size
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

            let image =
                Image::load_retained_image(&format!("{name}_{i}"), &buf.get_ref()[..pos as usize])
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
    map: HashMap<String, Image>,
    loader: Loader,
}

impl Cache {
    pub fn create(repaint: impl Fn() + Clone + Send + Sync + 'static) -> Self {
        Self {
            map: HashMap::default(),
            loader: Loader::spawn(repaint),
        }
    }

    pub fn get(&mut self, url: &str) -> Option<&Image> {
        match self.map.get(url) {
            Some(img) => Some(img),
            None => {
                self.loader.request(url);
                None
            }
        }
    }

    pub fn poll(&mut self) {
        for (k, v) in self.loader.produce.try_iter() {
            self.map.insert(k, v);
        }
    }
}

#[derive(Clone)]
struct Loader {
    submit: flume::Sender<String>,
    produce: flume::Receiver<(String, Image)>,
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
            return None;
        }

        resp.bytes().await.ok().map(|d| d.to_vec())
    }

    fn load(name: &str, data: Vec<u8>) -> anyhow::Result<Image> {
        // Check is svg
        if guess_svg(&data[..data.len().min(128)]) {
            Ok(Image::load_svg(name, &data).map(Image::Static)?)
        } else {
            let img = match image::guess_format(&data[..data.len().min(128)])
                .map_err(|err| anyhow::anyhow!("cannot guess format for: '{name}': {err}"))?
            {
                ImageFormat::Png => {
                    let dec = image::codecs::png::PngDecoder::new(&*data).map_err(|err| {
                        anyhow::anyhow!("expected png, got something else for '{name}': {err}")
                    })?;

                    if dec.is_apng() {
                        Animated::load_apng(name, &data).map(Image::Animated)?
                    } else {
                        Image::load_retained_image(name, &data).map(Image::Static)?
                    }
                }
                ImageFormat::Jpeg => Image::load_retained_image(name, &data).map(Image::Static)?,
                ImageFormat::Gif => Animated::load_gif(name, &data).map(Image::Animated)?,
                // TODO determine if its animated?
                ImageFormat::WebP => match data.get(44..48).filter(|bytes| bytes == b"ANMF") {
                    Some(_) => Animated::load_webp(name, &data).map(Image::Animated)?,
                    None => Image::load_retained_image(name, &data).map(Image::Static)?,
                },
                fmt => anyhow::bail!("unsupported format for '{name}': {fmt:?}"),
            };
            Ok(img)
        }
    }
}

fn guess_svg(buffer: &[u8]) -> bool {
    buffer.starts_with(&[0x3c, 0x3f, 0x78, 0x6d, 0x6c])
        || buffer.starts_with(&[0x3c, 0x73, 0x76, 0x67, 0x20])
}
