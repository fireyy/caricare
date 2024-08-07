use crate::config::ClientConfig;
use crate::Result;
use opendal::services::S3;
use opendal::Operator;
use std::sync::Arc;

pub(crate) fn create(config: &Arc<ClientConfig>) -> Result<Operator> {
    let mut builder = S3::default();
    builder.bucket(&config.bucket);
    builder.endpoint(&config.endpoint);
    builder.access_key_id(&config.access_key_id);
    builder.secret_access_key(&config.access_key_secret);
    let operator: Operator = Operator::new(builder)?.finish();

    Ok(operator)
}
