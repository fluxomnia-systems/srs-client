use serde::{Deserialize, Serialize};

fn default_vhost_key() -> String {
    String::new()
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Summary {
    #[serde(default)]
    pub urls: Option<Box<Urls>>,
    #[serde(default)]
    pub tests: Option<Tests>,
    #[serde(default)]
    pub ok: Option<bool>,
    #[serde(default)]
    pub now_ms: Option<u64>,
    #[serde(rename = "self", default)]
    pub self_: Option<SummarySelf>,
    #[serde(default)]
    pub system: Option<SummarySystem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SummarySelf {
    pub version: Option<String>,
    pub pid: Option<i64>,
    pub ppid: Option<i64>,
    pub argv: Option<String>,
    pub cwd: Option<String>,
    pub mem_kbyte: Option<i64>,
    pub mem_percent: Option<f64>,
    pub cpu_percent: Option<f64>,
    pub srs_uptime: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SummarySystem {
    #[serde(rename = "cpu_percent")]
    pub cpu_percent: Option<f64>,
    #[serde(rename = "disk_read_KBps")]
    pub disk_read_kbps: Option<i64>,
    #[serde(rename = "disk_write_KBps")]
    pub disk_write_kbps: Option<i64>,
    #[serde(rename = "disk_busy_percent")]
    pub disk_busy_percent: Option<f64>,
    #[serde(rename = "mem_ram_kbyte")]
    pub mem_ram_kbyte: Option<i64>,
    #[serde(rename = "mem_ram_percent")]
    pub mem_ram_percent: Option<f64>,
    #[serde(rename = "mem_swap_kbyte")]
    pub mem_swap_kbyte: Option<i64>,
    #[serde(rename = "mem_swap_percent")]
    pub mem_swap_percent: Option<f64>,
    pub cpus: Option<i64>,
    pub cpus_online: Option<i64>,
    pub uptime: Option<f64>,
    pub ilde_time: Option<f64>,
    pub load_1m: Option<f64>,
    pub load_5m: Option<f64>,
    pub load_15m: Option<f64>,
    pub net_sample_time: Option<u64>,
    pub net_recv_bytes: Option<u64>,
    pub net_send_bytes: Option<u64>,
    pub net_recvi_bytes: Option<u64>,
    pub net_sendi_bytes: Option<u64>,
    pub srs_sample_time: Option<u64>,
    pub srs_recv_bytes: Option<u64>,
    pub srs_send_bytes: Option<u64>,
    pub conn_sys: Option<i64>,
    pub conn_sys_et: Option<i64>,
    pub conn_sys_tw: Option<i64>,
    pub conn_sys_udp: Option<i64>,
    pub conn_srs: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(clippy::pub_underscore_fields)]
pub struct Tests {
    pub requests: String,
    pub errors: String,
    pub redirects: String,
    #[serde(rename = "[vhost]", default = "default_vhost_key")]
    pub vhost: String,
    #[serde(default = "default_vhost_key")]
    pub _vhost: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Urls {
    pub versions: String,
    pub summaries: String,
    pub rusages: String,
    pub self_proc_stats: String,
    pub system_proc_stats: String,
    pub meminfos: String,
    pub authors: String,
    pub features: String,
    pub requests: String,
    pub vhosts: String,
    pub streams: String,
    pub clients: String,
    pub raw: String,
    pub clusters: String,
    pub perf: String,
    pub tcmalloc: String,
}
