use crate::util::get_extension;
use aliyun_oss_client::{errors::OssError, file::File, object::ObjectList, Client, Query};
use md5;
use std::path::PathBuf;

pub enum UploadResult {
    Success(String),
    Error(String),
}

#[derive(Default, Clone)]
pub struct OssClient {
    path: String,
    url: String,
    client: Client,
}

impl OssClient {
    pub fn new() -> Result<Self, OssError> {
        simple_env_load::load_env_from(&[".env"]);
        let path = std::env::var("ALIYUN_BUCKET_PATH").unwrap_or("".to_string());
        let url = std::env::var("CDN_URL").unwrap_or("".to_string());

        let client = Client::from_env()?;

        Ok(Self { path, url, client })
    }

    // pub fn get_bucket_domain(&self) -> String {
    //     let bucket = String::from("https://") + &self.bucket + ".";
    //     let endpoint = self.endpoint.replace("https://", &bucket);
    //     endpoint
    // }

    pub fn get_file_url(&self, path: &String) -> String {
        format!("{}{path}", self.get_url())
        // self.client
        //     .get_endpoint_url()
        //     .join(&path)
        //     .unwrap()
        //     .to_string()
    }

    pub fn get_path(&self) -> &String {
        &self.path
    }

    pub fn get_url(&self) -> String {
        if self.url.is_empty() {
            self.client.get_bucket_url().to_string()
        } else {
            self.url.clone()
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub async fn put(&self, path: PathBuf) -> Result<String, OssError> {
        // let path = PathBuf::from(path);
        let path_clone = path.clone();
        let bucket_path = self.path.clone();
        let ext = get_extension(path_clone);
        let file_content = std::fs::read(path).unwrap();
        let key = format!("{}/{:x}.{}", bucket_path, md5::compute(&file_content), ext);
        let get_content_type = |content: &Vec<u8>| match infer::get(content) {
            Some(con) => Some(con.mime_type()),
            None => None,
        };
        let result = self
            .client()
            .put_content(file_content, key, get_content_type)
            .await;
        result
    }

    pub async fn put_multi(&self, paths: Vec<PathBuf>) -> Result<Vec<UploadResult>, OssError> {
        let mut results = vec![];
        for path in paths {
            match self.put(path).await {
                Ok(str) => results.push(UploadResult::Success(str)),
                Err(err) => results.push(UploadResult::Error(err.message())),
            }
        }

        Ok(results)
    }

    pub async fn get_list(&self, query: Query) -> Result<ObjectList, OssError> {
        // FIXME: client clone
        let result = self.client().clone().get_object_list(query).await;
        tracing::info!("{:?}", result);
        result
    }
}
