use std::io::Cursor;

use serde::Deserialize;
use thiserror::Error;

use crate::Result;

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct ServiceError {
    #[serde(rename = "Code", default)]
    pub(crate) code: String,
    #[serde(rename = "Message", default)]
    pub(crate) message: String,
    #[serde(rename = "RequestId", default)]
    pub(crate) request_id: String,
    #[serde(rename = "HostId", default)]
    pub(crate) host_id: String,
    #[serde(rename = "Endpoint", default)]
    pub(crate) endpoint: String,
}

impl ServiceError {
    pub(crate) fn try_from_xml(xml: &Vec<u8>) -> Result<Self> {
        let c = Cursor::new(xml);
        let e: ServiceError = quick_xml::de::from_reader(c)?;
        Ok(e)
    }
}

#[derive(Error, Debug)]
pub enum OSSError {
    //oss: service returned error: StatusCode=%d, ErrorCode=%s, ErrorMessage=\"%s\", RequestId=%s
    #[error("oss: service returned error: StatusCode={0}, ErrorCode={1}, ErrorMessage='{2}', RequestId={3}")]
    ServiceError(u16, String, String, String),
    #[error("{0}")]
    WithDescription(String),
}
