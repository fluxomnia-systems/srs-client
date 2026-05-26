//! SRS Client helps to
//!
//! Simplify to working process with [SRS] under Rust.
//!
//! [SRS]: https://ossrs.io
mod callback_api;
mod http_api;

pub use crate::{
    callback_api::{SrsCallbackEvent, SrsCallbackReq},
    http_api::{
        Audio, Client, Clusters, Hls, Kbps, Publish, SrsClient, SrsClientError, SrsClientResp,
        SrsClientRespData, Stream, Summary, Tests, Urls, Vhost, Video,
    },
};
