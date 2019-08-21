use serde_derive::{Deserialize, Serialize};

use paperclip::actix::api_v2_schema;

#[api_v2_schema]
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub message: String,
}
