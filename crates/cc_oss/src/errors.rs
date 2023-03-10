use quick_xml::Error as QxmlError;
use reqwest::header::InvalidHeaderName as HttpInvalidHeaderNameError;
use reqwest::header::InvalidHeaderValue as HttpInvalidHeaderValueError;
use reqwest::Error as ReqwestError;
use std::error::Error as StdError;
use std::io::Error as IoError;
// use std::string::FromUtf8Error;
use std::str::Utf8Error;

#[derive(Debug, Display)]
pub enum Error {
    Object(ObjectError),
    Io(IoError),
    String(Utf8Error),
    Reqwest(ReqwestError),
    Qxml(QxmlError),
    Http(HttpError),
}

impl Error {
    /// 返回 oss 服务端的错误信息
    pub fn message(self) -> String {
        match self {
            _ => self.to_string(),
        }
    }
}

#[derive(Debug, Display)]
pub enum HttpError {
    HttpInvalidHeaderValue(HttpInvalidHeaderValueError),
    HttpInvalidHeaderName(HttpInvalidHeaderNameError),
}

impl From<QxmlError> for Error {
    fn from(e: QxmlError) -> Error {
        Error::Qxml(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::Io(e)
    }
}

impl From<ReqwestError> for Error {
    fn from(e: ReqwestError) -> Error {
        Error::Reqwest(e)
    }
}

impl From<HttpInvalidHeaderValueError> for Error {
    fn from(e: HttpInvalidHeaderValueError) -> Error {
        Error::Http(HttpError::HttpInvalidHeaderValue(e))
    }
}

impl From<HttpInvalidHeaderNameError> for Error {
    fn from(e: HttpInvalidHeaderNameError) -> Error {
        Error::Http(HttpError::HttpInvalidHeaderName(e))
    }
}

impl From<Utf8Error> for Error {
    fn from(e: Utf8Error) -> Error {
        Error::String(e)
    }
}

#[derive(Debug, Display)]
pub enum ObjectError {
    #[display(fmt = "PUT ERROR: {}", msg)]
    PutError { msg: String },
    #[display(fmt = "GET ERROR: {}", msg)]
    GetError { msg: String },
    #[display(fmt = "COPY ERROR: {}", msg)]
    CopyError { msg: String },
    #[display(fmt = "DELETE ERROR: {}", msg)]
    DeleteError { msg: String },
    #[display(fmt = "HEAD ERROR: {}", msg)]
    HeadError { msg: String },
}

impl StdError for Error {}
