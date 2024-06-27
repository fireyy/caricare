use thiserror::Error;

#[derive(Error, Debug)]
pub enum OSSError {
    //oss: service returned error: StatusCode=%d, ErrorCode=%s, ErrorMessage=\"%s\", RequestId=%s
    #[error("oss: service returned error: StatusCode={0}, ErrorCode={1}, ErrorMessage='{2}', RequestId={3}")]
    ServiceError(u16, String, String, String),
    #[error("{0}")]
    WithDescription(String),
}
