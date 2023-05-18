use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum ServiceType {
    S3,
    Oss,
    Gcs,
    Azblob,
    #[default]
    S3Compatible,
}

impl Display for ServiceType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            ServiceType::S3 => write!(f, "Amazon Simple Storage"),
            ServiceType::Oss => write!(f, "Aliyun Object Storage"),
            ServiceType::Gcs => write!(f, "Google Cloud Storage"),
            ServiceType::Azblob => write!(f, "Azure Blob Storage"),
            ServiceType::S3Compatible => write!(f, "S3-Compatible Object Storage"),
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct Session {
    pub service: ServiceType,
    pub key_id: String,
    pub key_secret: String,
    pub endpoint: String,
    pub bucket: String,
    pub note: String,
}

impl Debug for Session {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("service", &self.service)
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
