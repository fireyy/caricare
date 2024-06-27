use crate::util::get_name_form_path;
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

pub type Params = BTreeMap<String, Option<String>>;
pub type Headers = HashMap<String, String>;

#[derive(Clone, Debug, Default)]
pub struct ListObjects {
    bucket_name: String,
    delimiter: String,
    prefix: String,
    start_after: String,
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
        start_after: String,
        max_keys: String,
        is_truncated: bool,
    ) -> Self {
        ListObjects {
            bucket_name,
            delimiter,
            prefix,
            start_after,
            max_keys,
            is_truncated,
            ..Default::default()
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

    pub fn start_after(&self) -> &str {
        &self.start_after
    }

    pub fn set_start_after(&mut self, start_after: String) {
        self.start_after = start_after;
    }

    pub fn max_keys(&self) -> &str {
        &self.max_keys
    }

    pub fn is_truncated(&self) -> bool {
        self.is_truncated
    }

    pub fn set_is_truncated(&mut self, is_truncated: bool) {
        self.is_truncated = is_truncated;
    }

    pub fn next_continuation_token(&self) -> &Option<String> {
        &self.next_continuation_token
    }

    pub fn set_next_continuation_token(&mut self, next_continuation_token: Option<String>) {
        self.next_continuation_token = next_continuation_token;
    }

    pub fn set_objects(&mut self, objects: Vec<Object>) {
        self.objects = objects;
    }

    pub fn set_common_prefixes(&mut self, common_prefixes: Vec<Object>) {
        self.common_prefixes = common_prefixes;
    }

    pub fn contents(&self) -> Vec<Object> {
        let mut contents = self.common_prefixes.clone();
        let mut objects = self.objects.clone();
        contents.append(&mut objects);
        contents
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
    last_modified: Option<DateTime<Utc>>,
    size: usize,
    etag: String,
    mine_type: String,
    storage_class: String,
    owner_id: String,
    owner_display_name: String,
    obj_type: ObjectType,
    pub selected: bool,
    url: String,
}

impl Object {
    pub fn new(key: &str, last_modified: Option<DateTime<Utc>>, size: usize) -> Self {
        Object {
            key: key.to_owned(),
            last_modified,
            size,
            ..Default::default()
        }
    }

    pub fn new_folder(key: &str) -> Self {
        Object {
            key: key.to_owned(),
            obj_type: ObjectType::Folder,
            ..Default::default()
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn last_modified(&self) -> Option<DateTime<Utc>> {
        self.last_modified
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn etag(&self) -> &str {
        &self.etag
    }

    pub fn mine_type(&self) -> &str {
        &self.mine_type
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

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn size_string(&self) -> String {
        if self.size.eq(&0) {
            "Folder".into()
        } else {
            bytesize::ByteSize(self.size as u64).to_string()
        }
    }

    pub fn date_string(&self) -> String {
        match self.last_modified {
            Some(date) => date.format("%Y-%m-%d %H:%M:%S").to_string(),
            None => "_".into(),
        }
    }
    pub fn is_file(&self) -> bool {
        self.obj_type == ObjectType::File
    }
    pub fn is_folder(&self) -> bool {
        self.obj_type == ObjectType::Folder
    }
    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }
    pub fn set_mine_type(&mut self, mine_type: String) {
        self.mine_type = mine_type;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum BucketACL {
    PublicReadWrite,
    PublicRead,
    #[default]
    Private,
}

#[derive(Clone, Debug, Default)]
pub struct Bucket {
    name: String,
    grant: BucketACL,
}

impl Bucket {
    pub fn new(name: String, grant: BucketACL) -> Self {
        Bucket { name, grant }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn grant(&self) -> &BucketACL {
        &self.grant
    }

    pub fn is_private(&self) -> bool {
        self.grant == BucketACL::Private
    }

    pub fn get_acl_from_str(text: &str) -> BucketACL {
        match text {
            "public-read-write" => BucketACL::PublicReadWrite,
            "public-read" => BucketACL::PublicRead,
            "private" => BucketACL::Private,
            _ => BucketACL::Private,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct ListObjectsV2Params {
    pub prefix: String,
    pub start_after: String,
    pub is_truncated: bool,
}

impl ListObjectsV2Params {
    pub fn new(prefix: String, start_after: String, is_truncated: bool) -> Self {
        ListObjectsV2Params {
            prefix,
            start_after,
            is_truncated,
        }
    }
}
