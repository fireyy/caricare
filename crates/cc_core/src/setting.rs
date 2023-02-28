use crate::store;
use serde::{Deserialize, Serialize};

fn default_page_limit() -> u16 {
    40
}

#[derive(Clone, Debug, Copy, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub enum ShowType {
    #[default]
    List,
    Thumb,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Setting {
    /// Page limit.
    #[serde(default = "default_page_limit")]
    pub page_limit: u16,
    /// Data list Show Type
    pub show_type: ShowType,
    /// Auto login
    pub auto_login: bool,
}

impl Default for Setting {
    fn default() -> Self {
        Self {
            page_limit: 40,
            show_type: ShowType::default(),
            auto_login: true,
        }
    }
}

impl Setting {
    pub fn load() -> Self {
        store::get_local_config::<Self>("config").unwrap_or_default()
    }

    pub fn store(&self) {
        store::set_local_config("config", self)
    }
}
