//! [HTTP Callback API][1] of [SRS] exposed by application.
//!
//! [SRS]: https://ossrs.io/
//! [1]: https://ossrs.io/lts/en-us/docs/v6/doc/http-callback

use std::net::IpAddr;

use super::SrsCallbackEvent;
use serde::{Deserialize, Serialize};

/// Request performed by [SRS] to [HTTP Callback API][1].
///
/// [SRS]: https://ossrs.io/
/// [1]: https://ossrs.io/lts/en-us/docs/v6/doc/http-callback
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SrsCallbackReq {
    /// ID of the [SRS] server
    ///
    /// [SRS]: https://ossrs.io/
    pub server_id: String,

    /// Event that [SRS] reports about.
    ///
    /// [SRS]: https://ossrs.io/
    pub action: SrsCallbackEvent,

    /// ID of [SRS] client that happened event is related to.
    ///
    /// [SRS]: https://ossrs.io/
    pub client_id: String,

    /// IP address of [SRS] client that happened event is related to.
    ///
    /// [SRS]: https://ossrs.io/
    pub ip: IpAddr,

    /// [SRS] `vhost` ([virtual host][1]) of RTMP stream that happened event is
    /// related to.
    ///
    /// [SRS]: https://ossrs.io/
    /// [1]: https://github.com/ossrs/srs/wiki/migrate_v4_EN_rtmp-url-vhost
    pub vhost: String,

    /// [SRS] `app` of RTMP stream that happened event is related to.
    ///
    /// [SRS]: https://ossrs.io/
    pub app: String,

    /// [SRS] `stream` of RTMP stream that happened event is related to.
    ///
    /// [SRS]: https://ossrs.io/
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,
    /// Event-specific SRS `service_id` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,

    /// Event-specific SRS `param` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,

    /// Event-specific SRS `tcUrl` value.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "tcUrl")]
    pub tc_url: Option<String>,

    /// Event-specific SRS `pageUrl` value.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "pageUrl")]
    pub page_url: Option<String>,

    /// Event-specific SRS `stream_url` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_url: Option<String>,

    /// Event-specific SRS `stream_id` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_id: Option<String>,

    /// Event-specific SRS `send_bytes` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub send_bytes: Option<i64>,

    /// Event-specific SRS `recv_bytes` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recv_bytes: Option<i64>,

    /// Event-specific SRS `cwd` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,

    /// Event-specific SRS `file` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,

    /// Event-specific SRS `duration` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    /// Event-specific SRS `url` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Event-specific SRS `m3u8` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub m3u8: Option<String>,

    /// Event-specific SRS `m3u8_url` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub m3u8_url: Option<String>,

    /// Event-specific SRS `seq_no` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq_no: Option<i64>,

    /// Additional fields introduced by newer SRS versions.
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}

impl SrsCallbackReq {
    /// Combine [`SrsCallbackReq::app`] and [`SrsCallbackReq::stream`] fields.
    /// Uses for tracing
    #[allow(dead_code)]
    #[must_use]
    pub fn app_stream(&self) -> String {
        if let Some(stream) = &self.stream {
            format!("{}/{}", self.app, stream)
        } else {
            self.app.clone()
        }
    }
}
