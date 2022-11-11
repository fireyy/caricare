use aliyun_oss_client::{errors::OssError, Client};
use md5;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime;

pub struct OSS {
    pub client: Arc<Client>,
    rt: runtime::Runtime,
    pub path: String,
    pub url: String,
}

impl OSS {
    pub fn new() -> Self {
        simple_env_load::load_env_from(&[".env"]);
        let path = std::env::var("ALIYUN_BUCKET_PATH").unwrap_or("".to_string());
        let url = std::env::var("CDN_URL").unwrap_or("".to_string());
        let client = match Client::from_env() {
            Ok(c) => c,
            Err(err) => panic!("{:?}", err),
        };
        let rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        Self {
            client: Arc::new(client),
            rt,
            path,
            url,
        }
    }

    pub fn put(
        &self,
        path: String,
        callback: impl 'static + Send + FnOnce(Result<String, OssError>),
    ) {
        let path = PathBuf::from(path);
        let path_clone = path.clone();
        let ext = path_clone.extension().and_then(OsStr::to_str).unwrap();
        let file_content = std::fs::read(path).unwrap();
        let client = Arc::clone(&self.client);
        let key = format!("{}/{:x}.{}", self.path, md5::compute(&file_content), ext);
        self.rt.spawn(async move {
            let result = client.put_content(file_content, &key).await;
            callback(result);
        });
    }
}
