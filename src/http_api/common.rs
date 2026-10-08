use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Kbps {
    pub recv_30s: i64,
    pub send_30s: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Hls {
    pub enabled: bool,
    pub fragment: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Publish {
    pub active: bool,
    pub cid: Option<String>,
}

#[allow(clippy::struct_field_names)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    pub major: i64,
    pub minor: i64,
    pub revision: i64,
    pub version: String,
}
