use quick_xml::{events::Event, Reader};
use std::collections::HashMap;

use crate::{
    oss::{ObjectMeta, RequestType},
    prelude::OSS,
};

use super::errors::{Error, ObjectError};

use async_trait::async_trait;
use bytes::Bytes;

#[derive(Clone, Debug)]
pub struct ListObjects {
    bucket_name: String,
    delimiter: String,
    prefix: String,
    marker: String,
    max_keys: String,
    is_truncated: bool,
    next_continuation_token: Option<String>,

    pub objects: Vec<Object>,
    pub common_prefixes: Vec<Object>,
}

impl ListObjects {
    pub fn new(
        bucket_name: String,
        delimiter: String,
        prefix: String,
        marker: String,
        max_keys: String,
        is_truncated: bool,
        next_continuation_token: Option<String>,

        objects: Vec<Object>,
        common_prefixes: Vec<Object>,
    ) -> Self {
        ListObjects {
            bucket_name,
            delimiter,
            prefix,
            marker,
            max_keys,
            is_truncated,
            next_continuation_token,

            objects,
            common_prefixes,
        }
    }

    pub fn bucket_name(&self) -> &str {
        &self.bucket_name
    }

    pub fn delimiter(&self) -> &str {
        &self.delimiter
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn marker(&self) -> &str {
        &self.marker
    }

    pub fn max_keys(&self) -> &str {
        &self.max_keys
    }

    pub fn is_truncated(&self) -> bool {
        self.is_truncated
    }

    pub fn next_continuation_token(&self) -> &Option<String> {
        &self.next_continuation_token
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ObjectType {
    #[default]
    File,
    Folder,
}

#[derive(Clone, Debug, Default)]
pub struct Object {
    key: String,
    last_modified: String,
    size: usize,
    etag: String,
    r#type: String,
    storage_class: String,
    owner_id: String,
    owner_display_name: String,
    obj_type: ObjectType,
    pub selected: bool,
}

impl Object {
    pub fn new(
        key: String,
        last_modified: String,
        size: usize,

        etag: String,
        r#type: String,
        storage_class: String,
        owner_id: String,
        owner_display_name: String,
    ) -> Self {
        Object {
            key,
            last_modified,
            size,
            etag,
            r#type,
            storage_class,
            owner_id,
            owner_display_name,
            obj_type: ObjectType::File,
            selected: false,
        }
    }

    pub fn new_folder(key: String) -> Self {
        Object {
            key,
            obj_type: ObjectType::Folder,
            ..Default::default()
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn last_modified(&self) -> &str {
        &self.last_modified
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn etag(&self) -> &str {
        &self.etag
    }

    pub fn r#type(&self) -> &str {
        &self.r#type
    }

    pub fn storage_class(&self) -> &str {
        &self.storage_class
    }

    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    pub fn owner_display_name(&self) -> &str {
        &self.owner_display_name
    }

    pub fn name(&self) -> String {
        get_name_form_path(&self.key)
    }

    pub fn obj_type(&self) -> &ObjectType {
        &self.obj_type
    }

    pub fn size_string(&self) -> String {
        if self.size.eq(&0) {
            "Folder".into()
        } else {
            bytesize::ByteSize(self.size as u64).to_string()
        }
    }

    pub fn date_string(&self) -> String {
        if self.last_modified.is_empty() {
            "-".into()
        } else {
            match chrono::DateTime::parse_from_rfc3339(&self.last_modified) {
                Ok(date) => date.format("%Y-%m-%d %H:%M:%S").to_string(),
                Err(_) => "_".into(),
            }
        }
    }
    pub fn is_file(&self) -> bool {
        self.obj_type == ObjectType::File
    }
    pub fn is_folder(&self) -> bool {
        self.obj_type == ObjectType::Folder
    }
}

#[async_trait]
pub trait ObjectAPI {
    async fn list_object<S, H, R>(&self, headers: H, resources: R) -> Result<ListObjects, Error>
    where
        S: AsRef<str>,
        H: Into<Option<HashMap<S, S>>> + Send,
        R: Into<Option<HashMap<S, Option<S>>>> + Send;

    async fn get_object<S1, S2, H, R>(
        &self,
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<Bytes, Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        H: Into<Option<HashMap<S2, S2>>> + Send,
        R: Into<Option<HashMap<S2, Option<S2>>>> + Send;

    async fn put_object<S1, S2, H, R>(
        &self,
        buf: &[u8],
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        H: Into<Option<HashMap<S2, S2>>> + Send,
        R: Into<Option<HashMap<S2, Option<S2>>>> + Send;

    async fn copy_object_from_object<S1, S2, S3, H, R>(
        &self,
        src: S1,
        dest: S2,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        S3: AsRef<str> + Send,
        H: Into<Option<HashMap<S3, S3>>> + Send,
        R: Into<Option<HashMap<S3, Option<S3>>>> + Send;

    async fn delete_object<S>(&self, object_name: S) -> Result<(), Error>
    where
        S: AsRef<str> + Send;

    async fn head_object<S>(&self, object_name: S) -> Result<ObjectMeta, Error>
    where
        S: AsRef<str> + Send;
}

#[async_trait]
impl ObjectAPI for OSS {
    async fn list_object<S, H, R>(&self, headers: H, resources: R) -> Result<ListObjects, Error>
    where
        S: AsRef<str>,
        H: Into<Option<HashMap<S, S>>> + Send,
        R: Into<Option<HashMap<S, Option<S>>>> + Send,
    {
        let (host, headers) =
            self.build_request(RequestType::Get, String::new(), headers, resources)?;

        let resp = self.http_client.get(host).headers(headers).send().await?;

        let xml_str = resp.text().await?;
        // tracing::debug!("XML: {}", xml_str);
        let mut result = Vec::new();
        let mut reader = Reader::from_str(xml_str.as_str());
        reader.trim_text(true);

        let mut bucket_name = String::new();
        let mut prefix = String::new();
        let mut marker = String::new();
        let mut max_keys = String::new();
        let mut delimiter = String::new();
        let mut is_truncated = false;

        let mut key = String::new();
        let mut last_modified = String::new();
        let mut etag = String::new();
        let mut r#type = String::new();
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
                    b"Marker" => marker = reader.read_text(e.name())?.to_string(),
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
                    b"Type" => r#type = reader.read_text(e.name())?.to_string(),
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
                            r#type.clone(),
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
                        marker,
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
    async fn get_object<S1, S2, H, R>(
        &self,
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<Bytes, Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        H: Into<Option<HashMap<S2, S2>>> + Send,
        R: Into<Option<HashMap<S2, Option<S2>>>> + Send,
    {
        let (host, headers) =
            self.build_request(RequestType::Get, object_name, headers, resources)?;

        let resp = self.http_client.get(&host).headers(headers).send().await?;

        if resp.status().is_success() {
            Ok(resp.bytes().await?)
        } else {
            Err(Error::Object(ObjectError::GetError {
                msg: format!("can not get object, status code: {}", resp.status()).into(),
            }))
        }
    }

    async fn put_object<S1, S2, H, R>(
        &self,
        buf: &[u8],
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        H: Into<Option<HashMap<S2, S2>>> + Send,
        R: Into<Option<HashMap<S2, Option<S2>>>> + Send,
    {
        let (host, headers) =
            self.build_request(RequestType::Put, object_name, headers, resources)?;

        let resp = self
            .http_client
            .put(&host)
            .headers(headers)
            .body(buf.to_owned())
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Error::Object(ObjectError::DeleteError {
                msg: format!(
                    "can not put object, status code, status code: {}",
                    resp.status()
                )
                .into(),
            }))
        }
    }

    async fn copy_object_from_object<S1, S2, S3, H, R>(
        &self,
        src: S1,
        dest: S2,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        S3: AsRef<str> + Send,
        H: Into<Option<HashMap<S3, S3>>> + Send,
        R: Into<Option<HashMap<S3, Option<S3>>>> + Send,
    {
        let (host, mut headers) = self.build_request(RequestType::Put, dest, headers, resources)?;
        headers.insert("x-oss-copy-source", src.as_ref().parse()?);

        let resp = self.http_client.put(&host).headers(headers).send().await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Error::Object(ObjectError::CopyError {
                msg: format!("can not copy object, status code: {}", resp.status()).into(),
            }))
        }
    }

    async fn delete_object<S>(&self, object_name: S) -> Result<(), Error>
    where
        S: AsRef<str> + Send,
    {
        let headers = HashMap::<String, String>::new();
        let (host, headers) =
            self.build_request(RequestType::Delete, object_name, Some(headers), None)?;

        let resp = self
            .http_client
            .delete(&host)
            .headers(headers)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Error::Object(ObjectError::DeleteError {
                msg: format!("can not delete object, status code: {}", resp.status()).into(),
            }))
        }
    }

    async fn head_object<S>(&self, object_name: S) -> Result<ObjectMeta, Error>
    where
        S: AsRef<str> + Send,
    {
        let (host, headers) = self.build_request(
            RequestType::Head,
            object_name,
            None::<HashMap<String, String>>,
            None,
        )?;

        let resp = self.http_client.head(&host).headers(headers).send().await?;

        if resp.status().is_success() {
            Ok(ObjectMeta::from_header_map(resp.headers())?)
        } else {
            Err(Error::Object(ObjectError::DeleteError {
                msg: format!("can not head object, status code: {}", resp.status()).into(),
            }))
        }
    }
}

fn get_name_form_path(path: &str) -> String {
    path.split('/')
        .filter(|k| !k.is_empty())
        .last()
        .unwrap_or("")
        .to_string()
}
