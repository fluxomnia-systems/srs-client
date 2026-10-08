//! SRS Client helps to
//!
//! Simplify to working process with [SRS] under Rust.
//!
//! [SRS]: https://ossrs.io
mod callback_api;
mod http_api;

pub use crate::{
    callback_api::{
        SrsCallbackEvent, SrsCallbackReq, SrsCallbackResponse, SrsForwardData, SrsForwardResponse,
    },
    http_api::{
        Audio, Authors, Client, ClusterQuery, Clusters, Hls, Kbps, Publish, RawConfig,
        ReloadStatus, RequestInfo, RtcRequest, RtcResponse, SdpSession, SignalResponse, SrsClient,
        SrsClientError, SrsClientResp, SrsClientRespData, SrsSignal, Stream, Summary, Tcmalloc,
        Tests, Urls, ValgrindCheck, ValgrindResponse, Version, Vhost, Video,
    },
};
