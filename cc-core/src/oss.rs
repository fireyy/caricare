use aliyun_oss_client::{errors::OssError, file::File, object::ObjectList, Client, Query};
use md5;
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Default, Debug, Clone)]

pub struct OssConfig {
    pub key_id: String,
    pub key_secret: String,
    pub endpoint: String,
    pub bucket: String,
    pub path: String,
    pub url: String,
}

impl OssConfig {
    pub fn new() -> Self {
        simple_env_load::load_env_from(&[".env"]);
        let path = std::env::var("ALIYUN_BUCKET_PATH").unwrap_or("".to_string());
        let url = std::env::var("CDN_URL").unwrap_or("".to_string());
        let key_id = std::env::var("ALIYUN_KEY_ID").unwrap_or("".to_string());
        let key_secret = std::env::var("ALIYUN_KEY_SECRET").unwrap_or("".to_string());
        let endpoint = std::env::var("ALIYUN_ENDPOINT").unwrap_or("".to_string());
        let bucket = std::env::var("ALIYUN_BUCKET").unwrap_or("".to_string());

        Self {
            key_id,
            key_secret,
            endpoint,
            bucket,
            path,
            url,
        }
    }

    pub fn get_bucket_domain(&self) -> String {
        let bucket = String::from("https://") + &self.bucket + ".";
        let endpoint = self.endpoint.replace("https://", &bucket);
        endpoint
    }

    pub fn get_file_url(&self, path: String) -> String {
        self.get_bucket_domain() + "/" + &path
    }

    pub fn client(&self) -> Client {
        let client = aliyun_oss_client::client(
            self.key_id.clone(),
            self.key_secret.clone(),
            self.endpoint.clone().try_into().unwrap(),
            self.bucket.clone().try_into().unwrap(),
        );

        client
    }

    pub async fn put(&self, path: String) -> Result<String, OssError> {
        let path = PathBuf::from(path);
        let path_clone = path.clone();
        let bucket_path = self.path.clone();
        let ext = path_clone.extension().and_then(OsStr::to_str).unwrap();
        let file_content = std::fs::read(path).unwrap();
        let client = self.client();
        let key = format!("{}/{:x}.{}", bucket_path, md5::compute(&file_content), ext);
        let get_content_type = |content: &Vec<u8>| match infer::get(content) {
            Some(con) => Some(con.mime_type()),
            None => None,
        };
        let result = client
            .put_content(file_content, &key, get_content_type)
            .await;
        result
    }

    pub async fn get_list(&self) -> Result<ObjectList, OssError> {
        let client = self.client();
        let path = self.path.clone();
        let mut query = Query::new();
        query.insert("prefix", path);
        let result = client.get_object_list(query).await;
        tracing::info!("{:?}", result);
        result
    }
}
