use crate::config::ClientConfig;
use crate::{CustomLayer, Result};
use opendal::services::Gcs;
use opendal::Operator;
use std::sync::Arc;

pub(crate) fn create(config: &Arc<ClientConfig>) -> Result<Operator> {
    let mut builder = Gcs::default();
    builder.bucket(&config.bucket);
    builder.endpoint(&config.endpoint);
    builder.credential(&config.access_key_secret);
    let operator: Operator = Operator::new(builder)?.layer(CustomLayer).finish();

    Ok(operator)
}
