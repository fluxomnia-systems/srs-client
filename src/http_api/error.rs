use derive_more::{Display, Error};
use reqwest::{Client, Error as ReqwestError, Response as ReqwestResponse};

/// Possible errors of performing requests to [SRS HTTP API][1].
///
/// [1]: https://ossrs.io/lts/en-us/docs/v6/doc/http-api
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Display, Error)]
pub enum SrsClientError {
    /// SRS returned an application error, even if HTTP status was successful.
    #[display(fmt = "SRS API error code: {_0}")]
    ApiError(#[error(not(source))] i64),

    /// The endpoint is unavailable or was replaced by the server's discovery index.
    #[display(fmt = "Unsupported SRS endpoint: {_0}")]
    UnsupportedEndpoint(#[error(not(source))] String),

    /// A caller supplied an invalid parameter.
    #[display(fmt = "Invalid argument: {_0}")]
    InvalidArgument(#[error(not(source))] &'static str),

    /// Performing HTTP request failed itself.
    #[display(fmt = "Failed to perform HTTP request: {_0}")]
    RequestFailed(ReqwestError),

    /// [SRS HTTP API][1] responded with a bad [`StatusCode`].
    ///
    /// [`StatusCode`]: reqwest::StatusCode
    /// [1]: https://ossrs.io/lts/en-us/docs/v6/doc/http-callback
    #[display(fmt = "SRS HTTP API responded with bad status: {_0}")]
    BadStatus(#[error(not(source))] reqwest::StatusCode),

    /// Performing deserialize of [SRS HTTP API][1] response
    ///
    /// [1]: https://ossrs.io/lts/en-us/docs/v6/doc/http-callback
    #[display(fmt = "Failed to perform deserialize request: {_0}")]
    DeserializeError(ReqwestError),

    /// Performing deserialize of buffered [SRS HTTP API][1] response
    ///
    /// [1]: https://ossrs.io/lts/en-us/docs/v6/doc/http-callback
    #[display(fmt = "Failed to perform deserialize request: {_0}")]
    JsonDeserializeError(serde_json::Error),

    /// Failed to build [`SrsClient`] client because incorrect base Url
    ///
    /// [`SrsClient`]: crate::SrsClient
    #[display(fmt = "Failed to parse base URL: {_0}")]
    IncorrectBaseUrl(url::ParseError),

    /// Failed to create [`SrsClient`] API Url
    ///
    /// [`SrsClient`]: crate::SrsClient
    #[display(fmt = "Failed to parse URL: {_0}")]
    IncorrectApiUrl(url::ParseError),

    /// SRS HTTP API returned a valid response with an unexpected shape.
    #[display(fmt = "SRS HTTP API returned unexpected response, expected: {_0}")]
    UnexpectedResponse(#[error(not(source))] &'static str),
}
