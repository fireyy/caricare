use crate::util::get_name_form_path;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use time::format_description::well_known::Rfc3339;
use time::{format_description, OffsetDateTime};

pub type Params = BTreeMap<String, Option<String>>;
pub type Headers = HashMap<String, String>;

pub(crate) trait Credentials: Send + Sync {
    fn access_key_id(&self) -> &str;
    fn access_key_secret(&self) -> &str;
    fn security_token(&self) -> &str;
}

impl Debug for dyn Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("access_key_id", &self.access_key_id().to_string())
            .field("access_key_secret", &self.access_key_secret().to_string())
            .field("security_token", &self.security_token().to_string())
            .finish()
    }
}

#[derive(Clone, Debug)]
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
        next_continuation_token: Option<String>,

        objects: Vec<Object>,
        common_prefixes: Vec<Object>,
    ) -> Self {
        ListObjects {
            bucket_name,
            delimiter,
            prefix,
            start_after,
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

    pub fn start_after(&self) -> &str {
        &self.start_after
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
    mine_type: String,
    storage_class: String,
    owner_id: String,
    owner_display_name: String,
    obj_type: ObjectType,
    pub selected: bool,
    url: String,
}

impl Object {
    pub fn new(
        key: String,
        last_modified: String,
        size: usize,

        etag: String,
        storage_class: String,
        owner_id: String,
        owner_display_name: String,
    ) -> Self {
        Object {
            key,
            last_modified,
            size,
            etag,
            mine_type: Default::default(),
            storage_class,
            owner_id,
            owner_display_name,
            obj_type: ObjectType::File,
            selected: false,
            url: Default::default(),
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
        if self.last_modified.is_empty() {
            "-".into()
        } else {
            match OffsetDateTime::parse(&self.last_modified, &Rfc3339) {
                Ok(date) => {
                    let format =
                        format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
                            .unwrap();
                    date.format(&format).unwrap()
                }
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

    pub fn from_str(text: &str) -> BucketACL {
        match text {
            "public-read-write" => BucketACL::PublicReadWrite,
            "public-read" => BucketACL::PublicRead,
            "private" => BucketACL::Private,
            _ => BucketACL::Private,
        }
    }
}
