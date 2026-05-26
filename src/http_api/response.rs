use crate::http_api::{
    client::Client, clusters::Clusters, common::Version, feature::FeaturesData, meminfos::MemInfos,
    rusages::Rusages, self_proc_stats::SelfProcStats, stream::Stream, summary::Summary,
    system_proc_stats::SystemProcStats, vhost::Vhost,
};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SrsClientResp {
    pub code: i64,
    #[serde(default)]
    pub server: String,
    #[serde(default)]
    pub service: String,
    #[serde(default)]
    pub pid: String,
    #[serde(flatten)]
    pub data: SrsClientRespData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EmptyData {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SrsClientRespData {
    Empty(EmptyData),
    Stream { stream: Stream },
    Streams { streams: Vec<Stream> },
    Client { client: Client },
    Clients { clients: Vec<Client> },
    Vhost { vhost: Vhost },
    Vhosts { vhosts: Vec<Vhost> },
    Summary(Summary),
    Summaries { data: Summary },
    Version { data: Version },
    Feature { data: FeaturesData },
    Rusages { data: Rusages },
    SelfProcStats { data: Box<SelfProcStats> },
    SystemProcStats { data: SystemProcStats },
    MemInfos { data: MemInfos },
    Clusters { data: Clusters },
}
