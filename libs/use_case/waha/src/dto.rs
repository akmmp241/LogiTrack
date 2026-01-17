use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendTextReq {
    pub chat_id: String,
    pub text: String,
    pub session: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WahaClientErrorResponse {
    pub message: String,
    pub status_code: i32,
}
