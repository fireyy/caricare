use crate::store;
use serde::{Deserialize, Serialize};

fn default_page_limit() -> u16 {
    40
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Setting {
    /// Page limit.
    #[serde(default = "default_page_limit")]
    pub page_limit: u16,
}

impl Setting {
    pub fn load() -> Self {
        store::get_local_config::<Self>("config").unwrap_or_default()
    }

    pub fn store(&self) {
        store::set_local_config("config", self)
    }
}
