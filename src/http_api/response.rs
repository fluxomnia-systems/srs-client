use super::management::{
    Authors, ReloadStatus, RequestInfo, SignalResponse, Tcmalloc, ValgrindResponse,
};
use crate::http_api::{
    client::Client, clusters::Clusters, common::Version, feature::FeaturesData, meminfos::MemInfos,
    rusages::Rusages, self_proc_stats::SelfProcStats, stream::Stream, summary::Summary,
    system_proc_stats::SystemProcStats, vhost::Vhost,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Serialize, Debug)]
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
    Requests { data: RequestInfo },
    Authors { data: Authors },
    Raw { http_api: Value },
    Reload { data: ReloadStatus },
    Tcmalloc { data: Tcmalloc },
    Signal { data: SignalResponse },
    Valgrind { data: ValgrindResponse },
}

// Select by endpoint-specific fields before decoding. Do not let permissive,
// optional-only models consume unrelated or malformed endpoint responses.
impl<'de> Deserialize<'de> for SrsClientRespData {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        let object = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("expected SRS response object"))?;
        macro_rules! decode {
            ($value:expr) => {
                serde_json::from_value($value.clone()).map_err(serde::de::Error::custom)?
            };
        }
        for key in ["stream", "streams", "client", "clients", "vhost", "vhosts"] {
            if let Some(item) = object.get(key) {
                return Ok(match key {
                    "stream" => Self::Stream {
                        stream: decode!(item),
                    },
                    "streams" => Self::Streams {
                        streams: decode!(item),
                    },
                    "client" => Self::Client {
                        client: decode!(item),
                    },
                    "clients" => Self::Clients {
                        clients: decode!(item),
                    },
                    "vhost" => Self::Vhost {
                        vhost: decode!(item),
                    },
                    _ => Self::Vhosts {
                        vhosts: decode!(item),
                    },
                });
            }
        }
        if object.contains_key("urls") {
            return Ok(Self::Summary(decode!(value)));
        }
        if let Some(config) = object.get("http_api") {
            if !config.is_object() {
                return Err(serde::de::Error::custom("expected http_api object"));
            }
            return Ok(Self::Raw {
                http_api: config.clone(),
            });
        }
        if let Some(data) = object.get("data") {
            return decode_data(data);
        }
        if object.is_empty() {
            return Ok(Self::Empty(EmptyData {}));
        }
        Err(serde::de::Error::custom("unknown SRS response shape"))
    }
}

fn decode_data<E: serde::de::Error>(data: &Value) -> Result<SrsClientRespData, E> {
    use SrsClientRespData as Data;
    macro_rules! decode {
        ($value:expr) => {
            serde_json::from_value($value.clone()).map_err(serde::de::Error::custom)?
        };
    }
    let fields = data
        .as_object()
        .ok_or_else(|| serde::de::Error::custom("expected SRS data object"))?;
    Ok(if fields.contains_key("version") {
        Data::Version {
            data: decode!(data),
        }
    } else if fields.contains_key("now_ms") {
        Data::Summaries {
            data: decode!(data),
        }
    } else if fields.contains_key("features") {
        Data::Feature {
            data: decode!(data),
        }
    } else if fields.contains_key("ru_utime") {
        Data::Rusages {
            data: decode!(data),
        }
    } else if fields.contains_key("comm") {
        Data::SelfProcStats {
            data: decode!(data),
        }
    } else if fields.contains_key("user") {
        Data::SystemProcStats {
            data: decode!(data),
        }
    } else if fields.contains_key("MemTotal") {
        Data::MemInfos {
            data: decode!(data),
        }
    } else if fields.contains_key("uri") {
        Data::Requests {
            data: decode!(data),
        }
    } else if fields.contains_key("license") {
        Data::Authors {
            data: decode!(data),
        }
    } else if fields.contains_key("rid") {
        Data::Reload {
            data: decode!(data),
        }
    } else if fields.contains_key("release_rate") {
        Data::Tcmalloc {
            data: decode!(data),
        }
    } else if fields.contains_key("signo") {
        Data::Signal {
            data: decode!(data),
        }
    } else if fields.contains_key("check") {
        Data::Valgrind {
            data: decode!(data),
        }
    } else if fields.contains_key("origin") && fields.get("query").is_some_and(Value::is_object) {
        Data::Clusters {
            data: decode!(data),
        }
    } else {
        return Err(serde::de::Error::custom("unknown SRS data shape"));
    })
}
