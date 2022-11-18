use anyhow::Context;
use egui_extras::RetainedImage;
use image::ImageFormat;
use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    io::Write,
    time::{Duration, Instant},
};
use tokio_stream::StreamExt;

pub trait Repaint: Clone + Send + Sync {
    fn repaint(&self) {}
}

impl Repaint for () {}

impl Repaint for egui::Context {
    fn repaint(&self) {
        self.request_repaint()
    }
}

pub struct ImageCache {
    map: HashMap<String, Image>,
    fetcher: ImageFetcher,
}

impl ImageCache {
    pub fn new(fetcher: ImageFetcher) -> Self {
        Self {
            map: HashMap::default(),
            fetcher,
        }
    }

    pub fn get(&mut self, url: &str) -> Option<&Image> {
        match self.map.get(url) {
            Some(img) => Some(img),
            None => {
                self.fetcher.request(url);
                None
            }
        }
    }

    pub fn poll(&mut self) {
        for (k, v) in self.fetcher.poll() {
            self.map.insert(k, v);
        }
    }
}

#[derive(Clone)]
pub struct ImageFetcher {
    submit: flume::Sender<String>,
    produce: flume::Receiver<(String, Image)>,
}

impl ImageFetcher {
    pub fn spawn(repaint: impl Repaint + 'static) -> Self {
        let (submit, submit_rx) = flume::unbounded::<String>();
        let (produce_tx, produce) = flume::unbounded();

        crate::runtime::spawn(async move {
            let mut seen = HashSet::<String>::new();
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
                    if let Some(data) = Self::fetch(client, &url).await {
                        tokio::task::spawn_blocking(move || match Self::load_image(&url, data) {
                            Ok(img) => {
                                repaint.repaint();
                                let _ = tx.send((url, img));
                            }
                            Err(err) => {
                                eprintln!("error: {err}")
                            }
                        });
                    }
                });
            }
        });

        Self { submit, produce }
    }

    fn request(&self, url: &str) {
        let _ = self.submit.send(url.to_string());
    }

    fn poll(&mut self) -> impl Iterator<Item = (String, Image)> + '_ {
        self.produce.try_iter()
    }

    async fn fetch(client: reqwest::Client, url: &str) -> Option<Vec<u8>> {
        let resp = client.get(url).send().await.ok()?;

        if resp.status().as_u16() == 404 {
            return None;
        }

        resp.bytes().await.ok().map(|v| v.to_vec())
    }

    fn load_image(url: &str, data: Vec<u8>) -> anyhow::Result<Image> {
        let img = match image::guess_format(&data[..data.len().min(128)])
            .map_err(|err| anyhow::anyhow!("cannot guess format for '{url}': {err}"))?
        {
            image::ImageFormat::Png => {
                let dec = image::codecs::png::PngDecoder::new(&*data).map_err(|err| {
                    anyhow::anyhow!("expected png, got something else for '{url}': {err}")
                })?;

                if dec.is_apng() {
                    AnimatedImage::load_apng(url, &data).map(Image::Animated)?
                } else {
                    Self::load_retained_image(url, &data).map(Image::Static)?
                }
            }
            image::ImageFormat::Jpeg => Self::load_retained_image(url, &data).map(Image::Static)?,
            image::ImageFormat::Gif => AnimatedImage::load_gif(url, &data).map(Image::Animated)?,
            fmt => anyhow::bail!("unsupported format for '{url}': {fmt:?}"),
        };

        Ok(img)
    }

    fn load_retained_image(url: &str, data: &[u8]) -> anyhow::Result<egui_extras::RetainedImage> {
        RetainedImage::from_image_bytes(url, data)
            .map_err(|err| anyhow::anyhow!("cannot load '{url}': {err}"))
    }
}

pub enum Image {
    Static(egui_extras::RetainedImage),
    Animated(AnimatedImage),
}

impl Image {
    pub fn show_size(&self, ui: &mut egui::Ui, size: egui::Vec2) {
        match self {
            Image::Static(img) => {
                img.show_size(ui, size);
            }
            Image::Animated(img) => {
                let dt = ui.ctx().input().stable_dt.min(0.1);
                if let Some((img, delay)) = img.frame(dt) {
                    img.show_size(ui, size);
                    ui.ctx().request_repaint_after(delay);
                }
            }
        }
    }
    pub fn size_vec2(&self) -> egui::Vec2 {
        match self {
            Image::Static(img) => img.size_vec2(),
            Image::Animated(_img) => egui::Vec2::new(1.0, 1.0), //TODO: Animated Size
        }
    }
}

pub struct AnimatedImage {
    frames: Vec<egui_extras::RetainedImage>,
    intervals: Vec<Duration>,
    position: Cell<usize>,
    last: Cell<Option<Instant>>,
}

impl AnimatedImage {
    fn frame(&self, dt: f32) -> Option<(&egui_extras::RetainedImage, Duration)> {
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

    fn load_apng(url: &str, data: &[u8]) -> anyhow::Result<Self> {
        use image::ImageDecoder as _;
        let dec = image::codecs::png::PngDecoder::new(data)?;
        anyhow::ensure!(dec.is_apng(), "expected an animated png");
        Self::load_frames(url, dec.total_bytes() as _, dec.apng())
    }

    fn load_gif(url: &str, data: &[u8]) -> anyhow::Result<Self> {
        use image::ImageDecoder as _;
        let dec = image::codecs::gif::GifDecoder::new(data)?;
        Self::load_frames(url, dec.total_bytes() as _, dec)
    }

    fn load_frames<'a>(
        name: &str,
        size_hint: usize,
        decoder: impl image::AnimationDecoder<'a>,
    ) -> anyhow::Result<Self> {
        let mut buf = std::io::Cursor::new(Vec::with_capacity(size_hint));
        let (mut frames, mut intervals) = (vec![], vec![]);

        for (i, frame) in decoder.into_frames().enumerate() {
            let frame = frame?;
            let delay = Duration::from(frame.delay());
            frame.buffer().write_to(&mut buf, ImageFormat::Png)?;
            buf.flush().expect("flush image transcode");
            let pos = buf.position();
            buf.set_position(0);
            let image = ImageFetcher::load_retained_image(
                &format!("{name}_{i}"),
                &buf.get_ref()[..pos as usize],
            )
            .with_context(|| anyhow::anyhow!("cannot decode frame {i}"))?;

            frames.push(image);
            intervals.push(delay);
        }

        Ok(Self {
            frames,
            intervals,
            position: Cell::new(0),
            last: Cell::new(None),
        })
    }
}

pub mod runtime {
    use std::future::Future;

    static TOKIO_HANDLE: once_cell::sync::OnceCell<tokio::runtime::Handle> =
        once_cell::sync::OnceCell::new();

    pub fn start() -> std::io::Result<()> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        let handle = rt.handle().clone();
        let _thread = std::thread::spawn(move || {
            rt.block_on(std::future::pending::<()>());
        });

        TOKIO_HANDLE.get_or_init(|| handle);

        Ok(())
    }

    pub fn enter_context(f: impl FnOnce()) {
        let _g = TOKIO_HANDLE.get().expect("initialization").enter();
        f();
    }

    pub fn spawn<T>(fut: impl Future<Output = T> + Send + Sync + 'static) -> flume::Receiver<T>
    where
        T: Send + Sync + 'static,
    {
        let (tx, rx) = flume::bounded(1); // not-sync
        enter_context(|| {
            tokio::task::spawn(async move {
                let res = fut.await;
                let _ = tx.send(res);
            });
        });
        rx
    }
}
