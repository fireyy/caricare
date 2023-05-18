use crate::config::ClientConfig;
use crate::{CustomLayer, Result};
use opendal::services::Oss;
use opendal::Operator;
use std::sync::Arc;

pub(crate) fn create(config: &Arc<ClientConfig>) -> Result<Operator> {
    let mut builder = Oss::default();
    builder.bucket(&config.bucket);
    builder.endpoint(&config.endpoint);
    builder.access_key_id(&config.access_key_id);
    builder.access_key_secret(&config.access_key_secret);
    let operator: Operator = Operator::new(builder)?.layer(CustomLayer).finish();

    Ok(operator)
}
