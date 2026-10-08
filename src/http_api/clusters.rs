use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Clusters {
    #[serde(default)]
    pub query: Option<ClustersQuery>,
    #[serde(default)]
    pub origin: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClustersQuery {
    #[serde(default)]
    pub ip: String,
    #[serde(default)]
    pub vhost: String,
    #[serde(default)]
    pub app: String,
    #[serde(default)]
    pub stream: String,
}
