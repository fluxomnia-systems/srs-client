//! [HTTP Callback][1] definitions of [SRS].
//!
//! [SRS]: https://ossrs.io/
//! [1]: https://ossrs.io/lts/en-us/docs/v6/doc/http-callback
//!
//! To work with callbacks you have to define callback which is receiving
//! [`SrsCallbackReq`] struct.
//!
//! Return `0` or a JSON object containing `code: 0` to accept an ordinary hook.
//! Forward-backend hooks require [`SrsForwardResponse`]. `on_hls_notify` is an
//! HTTP GET notification, not a JSON callback payload.
#![allow(unused_imports)]
mod event;
mod request;

pub use self::{event::SrsCallbackEvent, request::SrsCallbackReq};

/// JSON acknowledgement for an ordinary SRS callback.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SrsCallbackResponse {
    pub code: i64,
}

/// Response to an `on_forward` backend callback.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SrsForwardResponse {
    pub code: i64,
    pub data: SrsForwardData,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SrsForwardData {
    pub urls: Vec<String>,
}
