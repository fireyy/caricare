use crate::config::ClientConfig;
use crate::{CustomLayer, Result};
use opendal::services::Azblob;
use opendal::Operator;
use std::sync::Arc;

pub(crate) fn create(config: &Arc<ClientConfig>) -> Result<Operator> {
    let mut builder = Azblob::default();
    builder.container(&config.bucket);
    builder.endpoint(&config.endpoint);
    builder.account_name(&config.access_key_id);
    builder.account_key(&config.access_key_secret);
    let operator: Operator = Operator::new(builder)?.layer(CustomLayer).finish();

    Ok(operator)
}
