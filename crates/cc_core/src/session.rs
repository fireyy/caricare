use crate::error::CoreError;
use crate::regex;
use aliyun_oss_client::config::Config;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Default, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct Session {
    pub key_id: String,
    pub key_secret: String,
    pub endpoint: String,
    pub bucket: String,
    pub note: String,
}

impl Debug for Session {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("key_id", &self.key_id)
            .field("endpoint", &self.endpoint)
            .field("bucket", &self.bucket)
            .field("note", &self.note)
            .finish()
    }
}

impl Display for Session {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.key_id)?;
        if !self.bucket.is_empty() {
            write!(f, " on {}", self.bucket)?;
        }
        Ok(())
    }
}

impl Session {
    pub fn is_empty(&self) -> bool {
        self.key_id.is_empty()
    }

    pub fn config(&self) -> Result<Config, CoreError> {
        Ok(Config::try_new(
            self.key_id.clone(),
            self.key_secret.clone(),
            self.endpoint.clone(),
            self.bucket.clone(),
        )?)
    }

    pub fn key_secret_mask(&self) -> String {
        regex!(r"(?P<prefix>\w{3})(?P<replace_value>\w*)(?P<suffix>\w{3})")
            .replace_all(
                &self.key_secret,
                format!("{}{}{}", "$prefix", "****", "$suffix"),
            )
            .to_string()
    }
}
