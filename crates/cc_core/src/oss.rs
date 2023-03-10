use crate::log::LogItem;
use crate::util::get_name;
use crate::{CoreError, Session};
use cc_oss::object::Object as OssObject;
use cc_oss::prelude::*;
use cc_oss::{errors::Error, query::Query};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default, Clone)]
pub struct OssClient {
    // session: Session,
    path: String,
    url: String,
    client: OSS,
}

impl OssClient {
    pub fn new(session: &Session) -> Result<Self, CoreError> {
        let path = std::env::var("ALIYUN_BUCKET_PATH").unwrap_or("".to_string());
        let url = std::env::var("CDN_URL").unwrap_or("".to_string());
        // let config = session.clone().config()?;

        let client = OSS::new(
            session.key_id.clone(),
            session.key_secret.clone(),
            session.endpoint.clone(),
            session.bucket.clone(),
        );

        Ok(Self {
            path,
            url,
            client,
            // session: session.clone(),
        })
    }

    // pub fn get_bucket_domain(&self) -> String {
    //     let bucket = String::from("https://") + &self.bucket + ".";
    //     let endpoint = self.endpoint.replace("https://", &bucket);
    //     endpoint
    // }

    pub fn get_file_url(&self, path: &str) -> String {
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
            self.client.get_bucket_url()
        } else {
            self.url.clone()
        }
    }

    pub fn client(&self) -> &OSS {
        &self.client
    }

    pub fn get_bucket_name(&self) -> &str {
        self.client.bucket()
    }

    pub async fn put(&self, path: PathBuf, dest: &str) -> Result<(), Error> {
        let name = get_name(&path);
        let file_content = std::fs::read(path).unwrap();
        let key = format!("{}{}", dest, name);
        let content_length = file_content.len().to_string();
        let mut headers = HashMap::new();
        headers.insert("content-length", content_length.as_str());
        if let Some(con) = infer::get(&file_content) {
            headers.insert("content-type", con.mime_type());
        }
        let result = self
            .client
            .put_object(&file_content, key, headers, None)
            .await;
        result
    }

    pub async fn put_multi(
        &self,
        paths: Vec<PathBuf>,
        dest: String,
    ) -> Result<Vec<LogItem>, Error> {
        let mut results = vec![];
        for path in paths {
            match self.put(path, &dest).await {
                Ok(_) => results.push(LogItem::upload().with_success("upload success".into())),
                Err(err) => results.push(LogItem::upload().with_error(err.to_string())),
            }
        }

        Ok(results)
    }

    pub async fn get_list(self, query: Query) -> Result<ListObjects, Error> {
        tracing::debug!("Query: {:?}", query);

        let query = query.to_hashmap();

        let res: Result<ListObjects, Error> = self.client.list_object(None, query).await;

        tracing::debug!("Result: {:?}", res);

        res
    }

    pub async fn create_folder(self, path: String) -> Result<(), Error> {
        let path = if path.ends_with("/") {
            path
        } else {
            format!("{path}/")
        };
        tracing::debug!("Create folder: {}", path);
        let result = self
            .client
            .put_object(&[0], path, None::<HashMap<&str, &str>>, None)
            .await;

        tracing::info!("Result: {:?}", result);
        result
    }

    pub async fn delete_object(self, obj: OssObject) -> Result<(), Error> {
        let result = self.client.delete_object(obj.key()).await;
        tracing::info!("Result: {:?}", result);
        result
    }

    pub async fn delete_multi_object(self, obj: Vec<OssObject>) -> Result<(), Error> {
        let mut query = HashMap::new();
        query.insert("delete", None);

        let mut xml = vec![
            r#"<?xml version="1.0" encoding="UTF-8"?>"#.to_string(),
            "\n<Delete><Quiet>false</Quiet>".to_string(),
        ];
        for o in obj.iter() {
            xml.push(format!("<Object><Key>{}</Key></Object>", o.key()));
        }
        xml.push("</Delete>".to_string());
        let result = xml.join("");
        let result_clone = result.clone();

        let mut headers: HashMap<&str, &str> = HashMap::new();
        let len = result.len().to_string().to_owned();
        headers.insert("content-length", &len);

        let md5_digest = md5::compute(result.as_bytes());
        let md5_str = base64::encode(md5_digest.0);
        tracing::debug!("md5_str: {}", base64::encode(md5_digest.0));
        headers.insert("content-md5", &md5_str);
        headers.insert("content-type", "application/xml");
        // headers.insert("encoding-type", "url");

        let result = self
            .client
            .delete_multi_object(result_clone, headers, query)
            .await;
        tracing::info!("Result: {:?}", result);
        result
    }
}
