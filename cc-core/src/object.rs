use aliyun_oss_client::{
    decode::{RefineObject, RefineObjectList},
    errors::OssError,
    types::CONTINUATION_TOKEN,
    Query,
};
use bytesize::ByteSize;

fn get_name_form_path(path: &str) -> String {
    path.split('/')
        .filter(|k| !k.is_empty())
        .last()
        .unwrap_or("")
        .to_string()
}

#[derive(Clone, Default, Debug)]
pub enum OssObjectType {
    #[default]
    File,
    Folder,
}

#[derive(Clone, Default, Debug)]
pub struct OssObject {
    pub obj_type: OssObjectType,
    pub path: String,
    pub size: u64,
    pub last_modified: String,
}

impl RefineObject for OssObject {
    type Error = OssError;

    fn set_key(&mut self, key: &str) -> Result<(), Self::Error> {
        self.path = key.to_string();
        Ok(())
    }

    fn set_last_modified(&mut self, last_modified: &str) -> Result<(), Self::Error> {
        self.last_modified = last_modified.to_string();
        Ok(())
    }

    fn set_size(&mut self, size: &str) -> Result<(), Self::Error> {
        self.size = size.parse::<u64>().map_err(OssError::from)?;
        Ok(())
    }
}

impl OssObject {
    #[inline]
    pub fn path(&self) -> &String {
        &self.path
    }
    #[inline]
    pub fn size(&self) -> u64 {
        self.size
    }
    #[inline]
    pub fn last_modified(&self) -> &String {
        &self.last_modified
    }
    #[inline]
    pub fn name(&self) -> String {
        get_name_form_path(&self.path)
    }
    #[inline]
    pub fn size_string(&self) -> String {
        ByteSize(self.size).to_string()
    }
}

#[derive(Clone, Default, Debug)]
pub struct OssBucket {
    name: String,
    next_continuation_token: Option<String>,
    pub files: Vec<OssObject>,
    search_query: Query,
    pub common_prefixes: Vec<OssObject>,
}

impl RefineObjectList<OssObject> for OssBucket {
    type Error = OssError;

    fn set_name(&mut self, name: &str) -> Result<(), Self::Error> {
        self.name = name.to_string();
        Ok(())
    }

    fn set_list(&mut self, list: Vec<OssObject>) -> Result<(), Self::Error> {
        self.files = list;
        Ok(())
    }

    #[inline]
    fn set_common_prefix(
        &mut self,
        list: &Vec<std::borrow::Cow<'_, str>>,
    ) -> Result<(), Self::Error> {
        for val in list.iter() {
            self.common_prefixes.push(OssObject {
                obj_type: OssObjectType::Folder,
                path: val.to_string(),
                size: 0,
                last_modified: "".into(),
            });
        }
        Ok(())
    }

    #[inline]
    fn set_next_continuation_token(&mut self, token: Option<&str>) -> Result<(), Self::Error> {
        self.next_continuation_token = token.map(|t| t.to_owned());
        Ok(())
    }
}

impl OssBucket {
    pub fn next_query(&self) -> Option<Query> {
        match &self.next_continuation_token {
            Some(token) => {
                let mut search_query = self.search_query.clone();
                search_query.insert(CONTINUATION_TOKEN, token.to_owned());
                Some(search_query)
            }
            None => None,
        }
    }
}
