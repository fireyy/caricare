mod syntax_highlighting;

use std::collections::HashMap;

pub enum FileType {
    StaticImage(Vec<u8>),
    PlainText(Vec<u8>),
    Unknown,
}

impl std::fmt::Debug for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StaticImage(img) => f
                .debug_struct("StaticImage")
                .field("size", &img.len())
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
        matches!(self, Self::StaticImage(_))
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            Self::PlainText(data) => {
                if let Ok(text) = std::str::from_utf8(data) {
                    syntax_highlighting::code_view_ui(ui, text)
                } else {
                    ui.heading("Couldn't Preview File")
                }
            }
            _ => {
                ui.centered_and_justified(|ui| ui.heading("Couldn't Preview File"))
                    .inner
            }
        }
    }

    pub fn guess_type(data: Vec<u8>) -> Option<infer::Type> {
        infer::get(&data[..data.len().min(128)])
    }
}

pub struct Cache {
    map: HashMap<String, FileType>,
}

impl Cache {
    pub fn create() -> Self {
        Self {
            map: HashMap::default(),
        }
    }

    pub fn get(&mut self, url: &str) -> Option<&FileType> {
        self.map.get(url)
    }

    pub fn check(&self, url: &str) -> Option<&FileType> {
        self.map.get(url)
    }

    pub fn add(&mut self, name: &str, data: Vec<u8>) {
        tracing::debug!("Add file: {name}");
        self.map.insert(name.to_string(), FileType::Unknown);
        if let Ok(file) = load_type(data) {
            self.map.insert(name.to_string(), file);
        }
    }

    pub fn big_file(&mut self, name: &str) {
        tracing::debug!("Big file: {name}");
        self.map.insert(name.to_string(), FileType::Unknown);
    }

    pub fn replace(&mut self, old_key: &str, new_key: &str) {
        if let Some(v) = self.map.remove(old_key) {
            self.map.insert(new_key.to_string(), v);
        }
    }
}

fn load_type(data: Vec<u8>) -> anyhow::Result<FileType> {
    if let Some(format) = infer::get(&data[..data.len().min(128)]) {
        tracing::debug!("Load type: {:?}", format);
        let file = match format.mime_type() {
            "image/png" | "image/jpeg" | "image/gif" | "image/webp" | "image/svg+xml" => {
                FileType::StaticImage(data)
            }
            _ => FileType::PlainText(data),
        };
        Ok(file)
    } else {
        Ok(FileType::PlainText(data))
    }
}
