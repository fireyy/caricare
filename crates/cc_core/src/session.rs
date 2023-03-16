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

    pub fn key_secret_mask(&self) -> String {
        let mut str = self.key_secret.clone();
        let len = str.len() - 3;
        str.replace_range(4..len, "****");
        str
    }
}
