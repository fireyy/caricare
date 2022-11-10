use aliyun_oss_client::blocking::builder::ClientWithMiddleware;
use aliyun_oss_client::{client::Client, errors::OssError};
use md5;
use std::ffi::OsStr;
use std::path::PathBuf;

pub struct OSS {
    pub client: Client<ClientWithMiddleware>,
}

impl OSS {
    pub fn new() -> Self {
        simple_env_load::load_env_from(&[".env"]);
        let client = match Client::<ClientWithMiddleware>::from_env() {
            Ok(c) => c,
            Err(err) => panic!("{:?}", err),
        };

        Self { client }
    }

    pub fn put(&self, path: String) -> Result<String, OssError> {
        let path = PathBuf::from(path);
        let path_clone = path.clone();
        let ext = path_clone.extension().and_then(OsStr::to_str).unwrap();
        let file_content = std::fs::read(path).unwrap();
        let digest = md5::compute(&file_content);
        let result = self
            .client
            .put_content(file_content, &format!("catrun/test/{:x}.{}", digest, ext));

        result
    }
}
