// use aliyun_oss_client::config::InvalidConfig;
// use aliyun_oss_client::errors::OssError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    // #[error("OssError: {0}")]
    // OssError(#[from] OssError),
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
    // #[error("InvalidConfig: {0}")]
    // InvalidConfig(#[from] InvalidConfig),
    #[error("{0}")]
    Custom(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
