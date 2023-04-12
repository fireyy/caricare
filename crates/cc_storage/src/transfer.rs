use crossbeam_channel::unbounded;
use std::collections::BTreeMap;

pub type TransferSender = crossbeam_channel::Sender<TransferType>;

#[derive(Clone, Default, Debug)]
pub struct TransferProgressInfo {
    pub total_bytes: u64,
    pub transferred_bytes: u64,
}

impl TransferProgressInfo {
    pub fn rate(&self) -> f32 {
        (self.transferred_bytes as f64 / self.total_bytes as f64) as f32
    }
}

#[derive(Clone)]
pub enum TransferType {
    Download(String, TransferProgressInfo),
    Upload(String, TransferProgressInfo),
}

pub struct TransferManager {
    pub is_show: bool,
    pub t_type: String,
    downloads: BTreeMap<String, TransferProgressInfo>,
    uploads: BTreeMap<String, TransferProgressInfo>,
    pub filter: String,
    pub progress_tx: crossbeam_channel::Sender<TransferType>,
    pub progress_rx: crossbeam_channel::Receiver<TransferType>,
}

impl TransferManager {
    pub fn new() -> Self {
        let (progress_tx, progress_rx) = unbounded();
        Self {
            is_show: false,
            t_type: "download".into(),
            downloads: BTreeMap::new(),
            uploads: BTreeMap::new(),
            filter: String::new(),
            progress_tx,
            progress_rx,
        }
    }

    pub fn data(&self) -> &BTreeMap<String, TransferProgressInfo> {
        if self.is_upload() {
            &self.uploads
        } else {
            &self.downloads
        }
    }

    pub fn is_upload(&self) -> bool {
        self.t_type == "upload"
    }

    pub fn total(&self) -> usize {
        self.downloads.len() + self.uploads.len()
    }

    pub fn show(&mut self, t_type: &str) {
        self.is_show = true;
        self.t_type = t_type.to_string();
    }

    pub fn close(&mut self) {
        self.is_show = false;
    }

    pub fn poll(&mut self, repaint: impl Fn() + Clone + Send + Sync + 'static) {
        while let Ok(update) = self.progress_rx.try_recv() {
            match update {
                TransferType::Upload(key, item) => {
                    self.update_upload(key, item);
                    repaint();
                }
                TransferType::Download(key, item) => {
                    self.update_download(key, item);
                    repaint();
                }
            }
        }
    }

    fn update_download(&mut self, key: String, item: TransferProgressInfo) {
        // tracing::debug!("Download `{key}`… {}/{}", item.current, item.total);
        if item.transferred_bytes == item.total_bytes {
            tracing::debug!("Download Done!");
        }
        self.downloads.insert(key, item);
    }

    fn update_upload(&mut self, key: String, item: TransferProgressInfo) {
        // tracing::debug!("Upload `{key}`… {}/{}", item.current, item.total);
        if item.transferred_bytes == item.total_bytes {
            tracing::debug!("Upload Done!");
        }
        self.uploads.insert(key, item);
    }
}
