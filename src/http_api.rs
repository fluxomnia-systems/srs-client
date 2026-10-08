//! [HTTP API][1] definitions of [SRS].
//!
//! [SRS]: https://ossrs.io/
//! [1]: https://ossrs.io/lts/en-us/docs/v6/doc/http-api
#![allow(unused_imports)]

mod client;
mod clusters;
mod common;
mod error;
mod feature;
mod management;
mod meminfos;
mod response;
mod rtc;
mod rusages;
mod self_proc_stats;
mod stream;
mod summary;
mod system_proc_stats;
mod vhost;

pub use client::Client;
pub use clusters::Clusters;
pub use common::{Hls, Kbps, Publish, Version};
pub use error::SrsClientError;
pub use management::*;
pub use response::{EmptyData, SrsClientResp, SrsClientRespData};
pub use rtc::*;
pub use stream::{Audio, Stream, Video};
pub use summary::{Summary, Tests, Urls};
pub use vhost::Vhost;

use reqwest::{Client as ReqwestClient, Response as ReqwestResponse};
use serde_json::Value;
use url::Url;

/// Client for performing requests to [HTTP API][1] of spawned [SRS].
///
/// [SRS]: https://ossrs.io/
/// [1]: https://ossrs.io/lts/en-us/docs/v6/doc/http-api
#[derive(Clone, Debug)]
pub struct SrsClient {
    http_client: ReqwestClient,
    base_url: Url,
}

fn empty_success_response(code: i64) -> SrsClientResp {
    SrsClientResp {
        code,
        server: String::new(),
        service: String::new(),
        pid: String::new(),
        data: SrsClientRespData::Empty(EmptyData {}),
    }
}

impl SrsClient {
    /// Build [`SrsClient`] for future call to [HTTP API][1] API of spawned [SRS]. .
    ///
    /// # Errors
    ///
    /// If incorrect `base_url` passed
    ///
    /// [SRS]: https://ossrs.io/
    /// [1]: https://ossrs.io/lts/en-us/docs/v6/doc/http-api
    pub fn build<S: Into<String>>(base_url: S) -> Result<Self, SrsClientError> {
        Self::with_http_client(base_url, ReqwestClient::new())
    }

    /// Builds a client using a configured reqwest transport (authentication, TLS, timeouts).
    ///
    /// # Errors
    /// Returns an error if the URL is not an absolute HTTP(S) URL.
    pub fn with_http_client<S: Into<String>>(
        base_url: S,
        http_client: ReqwestClient,
    ) -> Result<Self, SrsClientError> {
        let base_url = Url::parse(&base_url.into())
            .and_then(|url| url.join("/api/v1/"))
            .map_err(SrsClientError::IncorrectBaseUrl)?;
        if !matches!(base_url.scheme(), "http" | "https") || base_url.host_str().is_none() {
            return Err(SrsClientError::InvalidArgument(
                "base URL must use HTTP or HTTPS",
            ));
        }
        Ok(Self {
            http_client,
            base_url,
        })
    }

    async fn get(&self, url: &str) -> Result<ReqwestResponse, SrsClientError> {
        self.http_client
            .get(
                self.base_url
                    .join(url)
                    .map_err(SrsClientError::IncorrectApiUrl)?,
            )
            .send()
            .await
            .map_err(SrsClientError::RequestFailed)
    }

    async fn delete(&self, url: &str) -> Result<ReqwestResponse, SrsClientError> {
        self.http_client
            .delete(
                self.base_url
                    .join(url)
                    .map_err(SrsClientError::IncorrectApiUrl)?,
            )
            .send()
            .await
            .map_err(SrsClientError::RequestFailed)
    }

    async fn process_resp(&self, resp: ReqwestResponse) -> Result<SrsClientResp, SrsClientError> {
        let path = resp.url().path().to_owned();
        let value = Self::json_response(resp).await?;
        if value.get("urls").is_some() && path != "/api/v1/" && path != "/api/v1" {
            return Err(SrsClientError::UnsupportedEndpoint(path));
        }
        serde_json::from_value(value).map_err(SrsClientError::JsonDeserializeError)
    }

    async fn json_response(resp: ReqwestResponse) -> Result<Value, SrsClientError> {
        if !resp.status().is_success() {
            return Err(SrsClientError::BadStatus(resp.status()));
        }
        let value: Value = resp
            .json()
            .await
            .map_err(SrsClientError::DeserializeError)?;
        Self::check_code(&value)?;
        Ok(value)
    }

    fn check_code(value: &Value) -> Result<(), SrsClientError> {
        match value.get("code").and_then(Value::as_i64) {
            Some(0) => Ok(()),
            Some(code) => Err(SrsClientError::ApiError(code)),
            None => Err(SrsClientError::UnexpectedResponse("integer SRS code")),
        }
    }

    async fn process_resp_allow_empty_data(
        &self,
        resp: ReqwestResponse,
    ) -> Result<SrsClientResp, SrsClientError> {
        if !resp.status().is_success() {
            return Err(SrsClientError::BadStatus(resp.status()));
        }
        if resp.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(empty_success_response(0));
        }
        let text = resp.text().await.map_err(SrsClientError::RequestFailed)?;
        let value: Value =
            serde_json::from_str(&text).map_err(SrsClientError::JsonDeserializeError)?;
        Self::check_code(&value)?;
        let response: SrsClientResp =
            serde_json::from_value(value).map_err(SrsClientError::JsonDeserializeError)?;
        if !matches!(response.data, SrsClientRespData::Empty(_)) {
            return Err(SrsClientError::UnexpectedResponse("empty success response"));
        }
        Ok(response)
    }

    fn resource_path(resource: &str, id: &str) -> Result<String, SrsClientError> {
        if id.is_empty()
            || !id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        {
            return Err(SrsClientError::InvalidArgument("invalid SRS resource ID"));
        }
        Ok(format!("{resource}/{id}"))
    }

    /// [Kicks off][1] a client connected to [SRS] server by its `id`.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    ///
    /// [SRS]: https://ossrs.io/
    /// [1]: https://ossrs.io/lts/en-us/docs/v6/doc/http-api#kickoff-client
    pub async fn kickoff_client<T: Into<String>>(
        &self,
        id: T,
    ) -> Result<SrsClientResp, SrsClientError> {
        let resp = self
            .delete(&Self::resource_path("clients", &id.into())?)
            .await?;
        self.process_resp_allow_empty_data(resp).await
    }

    /// Retrieves the server version.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_version(&self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("versions").await?;
        self.process_resp(resp).await
    }

    /// Retrieves the server summary.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_summaries(&self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("summaries").await?;
        self.process_resp(resp).await
    }

    /// Retrieves the server summary as typed data.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_summary(&self) -> Result<Option<Summary>, SrsClientError> {
        let response = self.get_summaries().await?;
        match response.data {
            SrsClientRespData::Summaries { data: summary } => Ok(Some(summary)),
            _ => Err(SrsClientError::UnexpectedResponse("summary")),
        }
    }

    /// Retrieves cluster-related information.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_clusters(&self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("clusters").await?;
        self.process_resp(resp).await
    }

    /// Retrieves the cluster information as typed summary data.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_clusters_summary(&self) -> Result<Option<Clusters>, SrsClientError> {
        let response = self.get_clusters().await?;
        match response.data {
            SrsClientRespData::Clusters { data: clusters } => Ok(Some(clusters)),
            _ => Err(SrsClientError::UnexpectedResponse("clusters")),
        }
    }

    /// Retrieves HTTP request-debug data.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_requests(&self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("tests/requests").await?;
        self.process_resp(resp).await
    }

    /// Legacy method retained for migration; always returns `UnsupportedEndpoint`.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    #[deprecated(
        note = "Use get_request_info; this method never returned the advertised endpoint data"
    )]
    pub fn get_requests_summary(
        &self,
    ) -> std::future::Ready<Result<Option<Summary>, SrsClientError>> {
        std::future::ready(Err(SrsClientError::UnsupportedEndpoint(
            "get_requests_summary; use get_request_info".to_owned(),
        )))
    }

    /// Retrieves RAW API configuration via `raw?rpc=raw`.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_configs(&self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("raw?rpc=raw").await?;
        self.process_resp(resp).await
    }

    /// Legacy method retained for migration; always returns `UnsupportedEndpoint`.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    #[deprecated(
        note = "Use get_raw_config; this method never returned the advertised endpoint data"
    )]
    pub fn get_configs_summary(
        &self,
    ) -> std::future::Ready<Result<Option<Summary>, SrsClientError>> {
        std::future::ready(Err(SrsClientError::UnsupportedEndpoint(
            "get_configs_summary; use get_raw_config".to_owned(),
        )))
    }

    /// Retrieves the SRS v1 API discovery index.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_api(&self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("").await?;
        self.process_resp(resp).await
    }

    /// Retrieves API-level docs as typed summary data.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_api_summary(&self) -> Result<Option<Summary>, SrsClientError> {
        let response = self.get_api().await?;
        match response.data {
            SrsClientRespData::Summary(summary) => Ok(Some(summary)),
            _ => Err(SrsClientError::UnexpectedResponse("api summary")),
        }
    }

    /// Legacy method retained for migration; always returns `UnsupportedEndpoint`.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    #[deprecated(note = "Use get_metrics; this method never returned the advertised endpoint data")]
    pub fn get_perf(&self) -> std::future::Ready<Result<SrsClientResp, SrsClientError>> {
        std::future::ready(Err(SrsClientError::UnsupportedEndpoint(
            "get_perf; use get_metrics".to_owned(),
        )))
    }

    /// Legacy method retained for migration; always returns `UnsupportedEndpoint`.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    #[deprecated(note = "Use get_metrics; this method never returned the advertised endpoint data")]
    pub fn get_perf_summary(&self) -> std::future::Ready<Result<Option<Summary>, SrsClientError>> {
        std::future::ready(Err(SrsClientError::UnsupportedEndpoint(
            "get_perf_summary; use get_metrics".to_owned(),
        )))
    }

    /// Retrieves typed allocator diagnostics when enabled in the server build.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_tcmalloc(&self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("tcmalloc").await?;
        self.process_resp(resp).await
    }

    /// Legacy method retained for migration; always returns `UnsupportedEndpoint`.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    #[deprecated(
        note = "Use get_tcmalloc_stats; this method never returned the advertised endpoint data"
    )]
    pub fn get_tcmalloc_summary(
        &self,
    ) -> std::future::Ready<Result<Option<Summary>, SrsClientError>> {
        std::future::ready(Err(SrsClientError::UnsupportedEndpoint(
            "get_tcmalloc_summary; use get_tcmalloc_stats".to_owned(),
        )))
    }

    /// Legacy method retained for migration; always returns `UnsupportedEndpoint`.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    #[deprecated(
        note = "Use SrsCallbackReq; this method never returned the advertised endpoint data"
    )]
    pub fn get_dvr(&self) -> std::future::Ready<Result<SrsClientResp, SrsClientError>> {
        std::future::ready(Err(SrsClientError::UnsupportedEndpoint(
            "get_dvr; use SrsCallbackReq".to_owned(),
        )))
    }

    /// Legacy method retained for migration; always returns `UnsupportedEndpoint`.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    #[deprecated(
        note = "Use SrsCallbackReq; this method never returned the advertised endpoint data"
    )]
    pub fn get_dvr_summary(&self) -> std::future::Ready<Result<Option<Summary>, SrsClientError>> {
        std::future::ready(Err(SrsClientError::UnsupportedEndpoint(
            "get_dvr_summary; use SrsCallbackReq".to_owned(),
        )))
    }

    /// Manages all vhosts or a specified vhost.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_vhosts(&self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("vhosts").await?;
        self.process_resp(resp).await
    }

    /// Manages a specified vhost.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_vhost<T: Into<String>>(&self, id: T) -> Result<SrsClientResp, SrsClientError> {
        let resp = self
            .get(&Self::resource_path("vhosts", &id.into())?)
            .await?;
        self.process_resp(resp).await
    }

    /// Retrieves all vhosts as a typed list.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_vhost_list(&self) -> Result<Vec<Vhost>, SrsClientError> {
        let response = self.get_vhosts().await?;
        match response.data {
            SrsClientRespData::Vhosts { vhosts } => Ok(vhosts),
            _ => Err(SrsClientError::UnexpectedResponse("vhosts")),
        }
    }

    /// Retrieves a specified vhost as a typed item.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_vhost_item<T: Into<String>>(
        &self,
        id: T,
    ) -> Result<Option<Vhost>, SrsClientError> {
        let response = match self.get_vhost(id).await {
            Ok(response) => response,
            Err(SrsClientError::ApiError(2014)) => return Ok(None),
            Err(error) => return Err(error),
        };
        match response.data {
            SrsClientRespData::Vhost { vhost } => Ok(Some(vhost)),
            _ => Err(SrsClientError::UnexpectedResponse("vhost")),
        }
    }

    /// Manages all streams or a specified stream.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_streams(&self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("streams").await?;
        self.process_resp(resp).await
    }

    /// Manages all streams using SRS pagination.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_streams_page(
        &self,
        start: i64,
        count: i64,
    ) -> Result<SrsClientResp, SrsClientError> {
        if start < 0 || count <= 0 {
            return Err(SrsClientError::InvalidArgument(
                "pagination requires start >= 0 and count > 0",
            ));
        }
        let resp = self
            .get(&format!("streams?start={start}&count={count}"))
            .await?;
        self.process_resp(resp).await
    }

    /// Manages a specified stream.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_stream<T: Into<String>>(
        &self,
        id: T,
    ) -> Result<SrsClientResp, SrsClientError> {
        let resp = self
            .get(&Self::resource_path("streams", &id.into())?)
            .await?;
        self.process_resp(resp).await
    }

    /// Retrieves all streams as a typed list.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_stream_list(&self) -> Result<Vec<Stream>, SrsClientError> {
        let response = self.get_streams().await?;
        match response.data {
            SrsClientRespData::Streams { streams } => Ok(streams),
            _ => Err(SrsClientError::UnexpectedResponse("streams")),
        }
    }

    /// Retrieves a paginated stream response as a typed list.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_stream_page_list(
        &self,
        start: i64,
        count: i64,
    ) -> Result<Vec<Stream>, SrsClientError> {
        let response = self.get_streams_page(start, count).await?;
        match response.data {
            SrsClientRespData::Streams { streams } => Ok(streams),
            _ => Err(SrsClientError::UnexpectedResponse("streams page")),
        }
    }

    /// Retrieves a specified stream as a typed item.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_stream_item<T: Into<String>>(
        &self,
        id: T,
    ) -> Result<Option<Stream>, SrsClientError> {
        let response = match self.get_stream(id).await {
            Ok(response) => response,
            Err(SrsClientError::ApiError(2048)) => return Ok(None),
            Err(error) => return Err(error),
        };
        match response.data {
            SrsClientRespData::Stream { stream } => Ok(Some(stream)),
            _ => Err(SrsClientError::UnexpectedResponse("stream")),
        }
    }

    /// Manages all clients or a specified client, default query top 10 clients.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_clients(&self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("clients").await?;
        self.process_resp(resp).await
    }

    /// Manages all clients using SRS pagination.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_clients_page(
        &self,
        start: i64,
        count: i64,
    ) -> Result<SrsClientResp, SrsClientError> {
        if start < 0 || count <= 0 {
            return Err(SrsClientError::InvalidArgument(
                "pagination requires start >= 0 and count > 0",
            ));
        }
        let resp = self
            .get(&format!("clients?start={start}&count={count}"))
            .await?;
        self.process_resp(resp).await
    }

    /// Manages a specified client.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_client<T: Into<String>>(
        &self,
        id: T,
    ) -> Result<SrsClientResp, SrsClientError> {
        let resp = self
            .get(&Self::resource_path("clients", &id.into())?)
            .await?;
        self.process_resp(resp).await
    }

    /// Retrieves all clients as a typed list.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_client_list(&self) -> Result<Vec<Client>, SrsClientError> {
        let response = self.get_clients().await?;
        match response.data {
            SrsClientRespData::Clients { clients } => Ok(clients),
            _ => Err(SrsClientError::UnexpectedResponse("clients")),
        }
    }

    /// Retrieves a paginated client response as a typed list.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_client_page_list(
        &self,
        start: i64,
        count: i64,
    ) -> Result<Vec<Client>, SrsClientError> {
        let response = self.get_clients_page(start, count).await?;
        match response.data {
            SrsClientRespData::Clients { clients } => Ok(clients),
            _ => Err(SrsClientError::UnexpectedResponse("clients page")),
        }
    }

    /// Retrieves a specified client as a typed item.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_client_item<T: Into<String>>(
        &self,
        id: T,
    ) -> Result<Option<Client>, SrsClientError> {
        let response = match self.get_client(id).await {
            Ok(response) => response,
            Err(SrsClientError::ApiError(2049)) => return Ok(None),
            Err(error) => return Err(error),
        };
        match response.data {
            SrsClientRespData::Client { client } => Ok(Some(client)),
            _ => Err(SrsClientError::UnexpectedResponse("client")),
        }
    }

    /// Retrieves the supported features of SRS.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_features(&self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("features").await?;
        self.process_resp(resp).await
    }

    /// Retrieves the rusage of SRS.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_rusages(&self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("rusages").await?;
        self.process_resp(resp).await
    }

    /// Retrieves the self process stats.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_self_proc_stats(&self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("self_proc_stats").await?;
        self.process_resp(resp).await
    }

    /// Retrieves the system process stats.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_system_proc_stats(&self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("system_proc_stats").await?;
        self.process_resp(resp).await
    }

    /// Retrieves the meminfo of system.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_meminfos(&self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("meminfos").await?;
        self.process_resp(resp).await
    }
}
