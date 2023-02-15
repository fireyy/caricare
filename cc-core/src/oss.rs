use crate::util::get_extension;
use crate::{OssBucket, OssObject};
use aliyun_oss_client::{
    errors::OssError,
    file::{FileAs, FileError},
    object::ObjectList,
    BucketName, Client, Query,
};
use md5;
use std::env;
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

    pub fn get_bucket_name(&self) -> String {
        self.client.get_bucket_base().name().to_string()
    }

    pub async fn put(&self, path: PathBuf) -> Result<String, FileError> {
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
            .client
            .put_content_as(file_content, key, get_content_type)
            .await;
        result
    }

    pub async fn put_multi(&self, paths: Vec<PathBuf>) -> Result<Vec<UploadResult>, OssError> {
        let mut results = vec![];
        for path in paths {
            match self.put(path).await {
                Ok(str) => results.push(UploadResult::Success(str)),
                Err(err) => results.push(UploadResult::Error(err.to_string())),
            }
        }

        Ok(results)
    }

    pub async fn get_list(self, query: Query) -> Result<OssBucket, OssError> {
        let mut bucket = OssBucket::default();
        let bucket_name = env::var("ALIYUN_BUCKET").unwrap();
        let init_file = || OssObject::default();

        tracing::debug!("Query: {:?}", query);

        let res: Result<_, OssError> = self
            .client
            .base_object_list(
                bucket_name.parse::<BucketName>().unwrap(),
                query,
                &mut bucket,
                init_file,
            )
            .await;

        res?;

        tracing::debug!("Result: {:?}", bucket);

        Ok(bucket)
    }

    pub async fn get_list2(self, query: Query) -> Result<ObjectList, OssError> {
        let result = self.client.get_object_list(query).await;
        tracing::info!("Result: {:?}", result);
        result
    }
}
