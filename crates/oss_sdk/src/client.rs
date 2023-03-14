use quick_xml::{events::Event, Reader};
use std::path::PathBuf;
use std::sync::Arc;

use crate::config::ClientConfig;
use crate::conn::{Conn, UrlMaker};
use crate::types::{Bucket, BucketACL, Headers, ListObjects, Object, Params};
use crate::util::{self, get_name};
use crate::Result;
use reqwest::header::HeaderMap;

#[derive(Clone)]
pub struct Client {
    pub(crate) config: Arc<ClientConfig>,
    pub(crate) conn: Conn,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            config: Default::default(),
        }
    }

    fn new(config: ClientConfig) -> Result<Client> {
        let client = reqwest::Client::builder()
            .http1_only()
            .timeout(config.timeout)
            .build()?;

        let um = UrlMaker::new(&config.endpoint, config.cname, config.http_proxy.is_some())?;
        let config = Arc::new(config);
        let conn = Conn::new(config.clone(), Arc::new(um), client)?;

        Ok(Client { conn, config })
    }

    pub fn get_bucket_url(&self) -> String {
        let url = &self.config.endpoint;
        let name_str = self.config.bucket.to_string();

        let mut name = String::from("https://");
        name.push_str(&name_str);
        name.push('.');

        let bucket_url = url.replace("https://", &name);
        bucket_url
    }

    pub async fn get_bucket_info(&self) -> Result<Bucket> {
        let mut query = Params::new();
        query.insert("bucketInfo".into(), None);

        let (data, _headers) = self
            .do_request(reqwest::Method::GET, "", Some(query), None, vec![])
            .await?;

        let xml_str = std::str::from_utf8(&data)?;
        tracing::debug!("XML: {}", xml_str);

        let mut reader = Reader::from_str(xml_str);
        reader.trim_text(true);

        let bucket_info;
        let mut bucket_name = String::new();
        let mut grant = BucketACL::default();

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"Name" => bucket_name = reader.read_text(e.to_end().name())?.to_string(),
                    b"Grant" => {
                        let text = reader.read_text(e.to_end().name())?;
                        grant = Bucket::from_str(&text);
                    }

                    _ => (),
                },

                Ok(Event::End(ref e)) => match e.name().as_ref() {
                    _ => (),
                },

                Ok(Event::Eof) => {
                    bucket_info = Bucket::new(bucket_name, grant);
                    break;
                } // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }
        }

        Ok(bucket_info)
    }

    pub async fn head_object(&self, object: impl AsRef<str>) -> Result<HeaderMap> {
        let object = object.as_ref();
        let (_, headers) = self
            .do_request(reqwest::Method::HEAD, object, None, None, vec![])
            .await?;

        tracing::debug!("Response header: {:?}", headers);

        Ok(headers)
    }

    pub async fn get_object(&self, object: impl AsRef<str>) -> Result<Vec<u8>> {
        let object = object.as_ref();
        let (resp, _headers) = self
            .do_request(reqwest::Method::GET, object, None, None, vec![])
            .await?;

        Ok(resp)
    }

    pub async fn delete_object(&self, object: impl AsRef<str>) -> Result<()> {
        let object = object.as_ref();
        let _ = self
            .do_request(reqwest::Method::DELETE, object, None, None, vec![])
            .await?;

        Ok(())
    }

    pub async fn delete_multi_object(self, obj: Vec<Object>) -> Result<()> {
        let mut query = Params::new();
        query.insert("delete".into(), None);

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

        let mut headers = Headers::new();
        let len = result.len().to_string().to_owned();
        headers.insert("content-length".into(), len);

        let md5_digest = md5::compute(result.as_bytes());
        let md5_str = base64::encode(md5_digest.0);
        tracing::debug!("md5_str: {}", base64::encode(md5_digest.0));
        headers.insert("content-md5".into(), md5_str);
        headers.insert("content-type".into(), "application/xml".into());

        tracing::debug!("Delete: query: {:?}, headers: {:?}", query, headers);

        let _ = self
            .do_request(
                reqwest::Method::POST,
                "",
                Some(query),
                Some(headers),
                result_clone.into_bytes(),
            )
            .await?;

        Ok(())
    }

    pub async fn list_v2(&self, query: Option<Params>) -> Result<ListObjects> {
        tracing::debug!("List object: {:?}", query);
        let (resp, _headers) = self
            .do_request(reqwest::Method::GET, "", query, None, vec![])
            .await?;

        let xml_str = std::str::from_utf8(&resp)?;
        tracing::debug!("XML: {}", xml_str);
        let mut result = Vec::new();
        let mut reader = Reader::from_str(xml_str);
        reader.trim_text(true);

        let mut bucket_name = String::new();
        let mut prefix = String::new();
        let mut start_after = String::new();
        let mut max_keys = String::new();
        let mut delimiter = String::new();
        let mut is_truncated = false;

        let mut key = String::new();
        let mut last_modified = String::new();
        let mut etag = String::new();
        let mut size = 0usize;
        let mut storage_class = String::new();
        let mut owner_id = String::new();
        let mut owner_display_name = String::new();
        let mut next_continuation_token = None;

        let mut is_common_pre = false;
        let mut prefix_vec = Vec::new();

        let list_objects;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"CommonPrefixes" => {
                        is_common_pre = true;
                    }
                    b"Name" => bucket_name = reader.read_text(e.name())?.to_string(),
                    b"Prefix" => {
                        if is_common_pre {
                            let object = Object::new_folder(
                                reader.read_text(e.to_end().name())?.to_string(),
                            );
                            prefix_vec.push(object);
                        } else {
                            prefix = reader.read_text(e.name())?.to_string();
                        }
                    }
                    b"StartAfter" => start_after = reader.read_text(e.name())?.to_string(),
                    b"MaxKeys" => max_keys = reader.read_text(e.name())?.to_string(),
                    b"Delimiter" => delimiter = reader.read_text(e.name())?.to_string(),
                    b"IsTruncated" => {
                        is_truncated = reader.read_text(e.name())?.to_string() == "true"
                    }
                    b"NextContinuationToken" => {
                        let nc_token = reader.read_text(e.name())?.to_string();
                        next_continuation_token = if nc_token.len() > 0 {
                            Some(nc_token)
                        } else {
                            None
                        };
                    }
                    b"Contents" => {
                        // do nothing
                    }
                    b"Key" => key = reader.read_text(e.name())?.to_string(),
                    b"LastModified" => last_modified = reader.read_text(e.name())?.to_string(),
                    b"ETag" => etag = reader.read_text(e.name())?.to_string(),
                    b"Size" => size = reader.read_text(e.name())?.parse::<usize>().unwrap(),
                    b"StorageClass" => storage_class = reader.read_text(e.name())?.to_string(),
                    b"Owner" => {
                        // do nothing
                    }
                    b"ID" => owner_id = reader.read_text(e.name())?.to_string(),
                    b"DisplayName" => owner_display_name = reader.read_text(e.name())?.to_string(),

                    _ => (),
                },

                Ok(Event::End(ref e)) => match e.name().as_ref() {
                    b"CommonPrefixes" => {
                        is_common_pre = false;
                    }
                    b"Contents" => {
                        let object = Object::new(
                            key.clone(),
                            last_modified.clone(),
                            size,
                            etag.clone(),
                            storage_class.clone(),
                            owner_id.clone(),
                            owner_display_name.clone(),
                        );
                        result.push(object);
                    }
                    _ => (),
                },

                Ok(Event::Eof) => {
                    list_objects = ListObjects::new(
                        bucket_name,
                        delimiter,
                        prefix,
                        start_after,
                        max_keys,
                        is_truncated,
                        next_continuation_token,
                        result,
                        prefix_vec,
                    );
                    break;
                } // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }
        }
        Ok(list_objects)
    }

    pub async fn create_folder(&self, path: String) -> Result<()> {
        let path = if path.ends_with("/") {
            path
        } else {
            format!("{path}/")
        };
        tracing::debug!("Create folder: {}", path);

        let mut headers = Headers::new();
        headers.insert("content-length".into(), 0.to_string());

        let _ = self
            .do_request(reqwest::Method::PUT, &path, None, Some(headers), vec![])
            .await?;

        Ok(())
    }

    pub async fn put(&self, path: PathBuf, dest: &str) -> Result<()> {
        let name = get_name(&path);
        let file_content = std::fs::read(path).unwrap();
        let key = format!("{}{}", dest, name);
        let content_length = file_content.len().to_string();
        let mut headers = Headers::new();
        headers.insert("content-length".into(), content_length);
        if let Some(con) = infer::get(&file_content) {
            headers.insert("content-type".into(), con.mime_type().to_string());
        }

        let _ = self
            .do_request(
                reqwest::Method::PUT,
                &key,
                None,
                Some(headers),
                file_content,
            )
            .await?;

        Ok(())
    }

    pub async fn put_multi(&self, paths: Vec<PathBuf>, dest: String) -> Result<Vec<String>> {
        let mut results = vec![];
        for path in paths {
            match self.put(path, &dest).await {
                Ok(_) => results.push("upload success".into()),
                Err(err) => results.push(err.to_string()),
            }
        }

        Ok(results)
    }

    pub fn signature_url(
        &self,
        object: &str,
        expire: i64,
        params: Option<Params>,
    ) -> Result<String> {
        self.conn.signature_url(object, expire, params)
    }

    async fn do_request(
        &self,
        method: reqwest::Method,
        object: &str,
        query: Option<Params>,
        headers: Option<Headers>,
        data: Vec<u8>,
    ) -> Result<(Vec<u8>, reqwest::header::HeaderMap)> {
        util::check_bucket_name(&self.config.bucket)?;

        self.conn
            .execute(method, object, query, headers, data, 0)
            .await
    }
}

pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.endpoint = endpoint.into();
        self
    }

    pub fn access_key(mut self, key: impl Into<String>) -> Self {
        self.config.access_key_id = key.into();
        self
    }

    pub fn access_secret(mut self, secret: impl Into<String>) -> Self {
        self.config.access_key_secret = secret.into();
        self
    }

    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.config.bucket = bucket.into();
        self
    }

    pub fn build(self) -> Result<Client> {
        util::check_bucket_name(&self.config.bucket)?;
        Client::new(self.config)
    }
}
