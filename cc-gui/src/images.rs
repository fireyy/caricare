use cc_core::tracing;
use egui_extras::RetainedImage;
use std::collections::HashSet;

use cached_network_image::{
    FetchImage, FetchQueue, Image, ImageCache, ImageKind, ImageStore, Uuid,
};
use directories::ProjectDirs;

pub struct NetworkImages {
    pub image_store: ImageStore<Image>,
    pub fetch_queue: FetchQueue<Image>,
    pub caches: ImageCache,
    pub requested_images: HashSet<String>,
}
impl NetworkImages {
    pub fn new(ctx: egui::Context) -> Self {
        let path = ProjectDirs::from("com", "fireyy", "Caricare")
            .map(|proj_dirs| proj_dirs.config_dir().to_path_buf());
        let image_store = ImageStore::<Image>::new(path);
        Self {
            image_store: image_store.clone(),
            fetch_queue: FetchQueue::create(ctx, image_store),
            caches: ImageCache::default(),
            requested_images: HashSet::new(),
        }
    }

    pub fn add(&mut self, img: String) {
        if !self.requested_images.insert(img.clone()) {
            return;
        }
        self.fetch_queue.fetch(self.gen_image(img));
    }

    pub fn get_image(&self, url: String) -> Option<&RetainedImage> {
        if let Some(img_id) = self.image_store.get_id(&url) {
            self.caches.get_id(img_id)
        } else {
            None
        }
    }

    pub fn try_fetch(&mut self) {
        let (image, data) = match self.fetch_queue.try_next() {
            Some((image, data)) => (image, data),
            _ => return,
        };

        let images = &mut self.caches;
        if images.has_id(image.id) {
            return;
        }

        match RetainedImage::from_image_bytes(image.url(), &data) {
            Ok(img) => {
                images.add(image.id, img);
                let _ = self.requested_images.remove(&image.url);
                self.image_store.add(&image, &(), &data);
            }
            Err(err) => {
                tracing::error!("cannot create ({}) {} : {err}", image.id, image.url())
            }
        }
    }

    fn gen_image(&self, url: String) -> Image {
        let uuid = self
            .image_store
            .get_id(&url) //
            .unwrap_or_else(Uuid::new_v4);

        Image {
            id: uuid,
            kind: ImageKind::Display,
            url,
            meta: (),
        }
    }
}
