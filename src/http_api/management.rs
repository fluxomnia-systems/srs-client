//! Management response models and operations for SRS 6 and newer.
use super::{Clusters, SrsClient, SrsClientError, SrsClientResp, SrsClientRespData};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestInfo {
    pub uri: String,
    pub path: String,
    #[serde(rename = "METHOD")]
    pub method: String,
    pub headers: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Authors {
    pub license: String,
    pub contributors: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// RAW API configuration, preserving version-specific configuration fields.
#[derive(Debug, Serialize, Deserialize)]
pub struct RawConfig {
    pub http_api: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReloadStatus {
    pub err: i64,
    pub msg: String,
    pub state: i64,
    pub rid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tcmalloc {
    pub query: Value,
    pub release_rate: f64,
    pub generic: BTreeMap<String, u64>,
    pub tcmalloc: BTreeMap<String, u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignalResponse {
    pub signal: String,
    pub signo: i64,
    pub help: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValgrindResponse {
    pub check: String,
    pub call: String,
    pub help: String,
    pub see: String,
}

/// SRS process signals; availability depends on the server build.
#[derive(Clone, Copy, Debug, Serialize)]
pub enum SrsSignal {
    #[serde(rename = "SIGHUP")]
    Reload,
    #[serde(rename = "SIGUSR1")]
    ReopenLog,
    #[serde(rename = "SIGUSR2")]
    Upgrade,
    #[serde(rename = "SIGTERM")]
    FastQuit,
    #[serde(rename = "SIGQUIT")]
    GracefulQuit,
    #[serde(rename = "SIGABRT")]
    Abort,
    #[serde(rename = "SIGINT")]
    Interrupt,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ValgrindCheck {
    Full,
    Added,
    Changed,
    New,
    Quick,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ClusterQuery {
    pub ip: String,
    pub vhost: String,
    pub app: String,
    pub stream: String,
    pub coworker: String,
}

impl SrsClient {
    /// Reads request-debug data from the actual `/api/v1/tests/requests` handler.
    ///
    /// # Errors
    /// Returns transport, SRS application, or response-shape errors.
    pub async fn get_request_info(&self) -> Result<RequestInfo, SrsClientError> {
        match self.get_requests().await?.data {
            SrsClientRespData::Requests { data } => Ok(data),
            _ => Err(SrsClientError::UnexpectedResponse("request info")),
        }
    }

    /// Reads the server's license and contributor information.
    ///
    /// # Errors
    /// Returns transport, SRS application, or response-shape errors.
    pub async fn get_authors(&self) -> Result<Authors, SrsClientError> {
        match self.process_resp(self.get("authors").await?).await?.data {
            SrsClientRespData::Authors { data } => Ok(data),
            _ => Err(SrsClientError::UnexpectedResponse("authors")),
        }
    }

    /// Reads RAW API configuration, including enabled/reload permissions.
    ///
    /// # Errors
    /// Returns transport, SRS application, or response-shape errors.
    pub async fn get_raw_config(&self) -> Result<RawConfig, SrsClientError> {
        match self.get_configs().await?.data {
            SrsClientRespData::Raw { http_api } => Ok(RawConfig { http_api }),
            _ => Err(SrsClientError::UnexpectedResponse("RAW config")),
        }
    }

    /// Requests a configuration reload. Supported reload settings depend on SRS version.
    ///
    /// # Errors
    /// Returns an SRS application error when RAW/reload is disabled, or a transport/shape error.
    pub async fn reload(&self) -> Result<(), SrsClientError> {
        self.process_resp_allow_empty_data(self.get("raw?rpc=reload").await?)
            .await?;
        Ok(())
    }

    /// Reads reload progress; `err` reports the asynchronous reload result.
    ///
    /// # Errors
    /// Returns transport, SRS application, or response-shape errors.
    pub async fn get_reload_status(&self) -> Result<ReloadStatus, SrsClientError> {
        match self
            .process_resp(self.get("raw?rpc=reload-fetch").await?)
            .await?
            .data
        {
            SrsClientRespData::Reload { data } => Ok(data),
            _ => Err(SrsClientError::UnexpectedResponse("reload status")),
        }
    }

    /// Queries origin-cluster discovery with URL-encoded parameters.
    ///
    /// # Errors
    /// Returns transport, SRS application, or response-shape errors.
    pub async fn query_clusters(&self, query: &ClusterQuery) -> Result<Clusters, SrsClientError> {
        let response = self
            .http_client
            .get(
                self.base_url
                    .join("clusters")
                    .map_err(SrsClientError::IncorrectApiUrl)?,
            )
            .query(query)
            .send()
            .await
            .map_err(SrsClientError::RequestFailed)?;
        match self.process_resp(response).await?.data {
            SrsClientRespData::Clusters { data } => Ok(data),
            _ => Err(SrsClientError::UnexpectedResponse("clusters")),
        }
    }

    /// Reads Prometheus metrics. The SRS exporter must be enabled.
    ///
    /// # Errors
    /// Returns an SRS application error for a disabled exporter, or transport/shape errors.
    pub async fn get_metrics(&self) -> Result<String, SrsClientError> {
        self.text_response("/metrics").await
    }

    /// Reads allocator statistics as text; requires an SRS tcmalloc build.
    ///
    /// # Errors
    /// Returns `UnsupportedEndpoint` if the server falls back to API discovery.
    pub async fn get_tcmalloc_stats(&self) -> Result<String, SrsClientError> {
        self.text_response("tcmalloc?page=summary").await
    }

    async fn text_response(&self, path: &str) -> Result<String, SrsClientError> {
        let response = self.get(path).await?;
        if !response.status().is_success() {
            return Err(SrsClientError::BadStatus(response.status()));
        }
        let text = response
            .text()
            .await
            .map_err(SrsClientError::RequestFailed)?;
        if let Ok(value) = serde_json::from_str::<Value>(&text) {
            Self::check_code(&value)?;
            return Err(SrsClientError::UnsupportedEndpoint(path.to_owned()));
        }
        if text.trim().is_empty()
            || (path == "/metrics"
                && !text
                    .lines()
                    .any(|line| line.starts_with("# HELP") || line.starts_with("# TYPE")))
        {
            return Err(SrsClientError::UnexpectedResponse(
                "metrics or allocator text",
            ));
        }
        Ok(text)
    }

    /// Requests a process signal. Quit/abort variants can stop the SRS server.
    ///
    /// # Errors
    /// Returns an error if the build lacks signal support or the request fails.
    pub async fn signal(&self, signal: SrsSignal) -> Result<SignalResponse, SrsClientError> {
        match self.diagnostic("signal", "signo", signal).await?.data {
            SrsClientRespData::Signal { data } => Ok(data),
            _ => Err(SrsClientError::UnexpectedResponse("signal")),
        }
    }

    /// Requests a leak check from an SRS build compiled with Valgrind support.
    ///
    /// # Errors
    /// Returns an error if the build lacks Valgrind support or the request fails.
    pub async fn valgrind(&self, check: ValgrindCheck) -> Result<ValgrindResponse, SrsClientError> {
        match self.diagnostic("valgrind", "check", check).await?.data {
            SrsClientRespData::Valgrind { data } => Ok(data),
            _ => Err(SrsClientError::UnexpectedResponse("valgrind")),
        }
    }

    async fn diagnostic<T: Serialize>(
        &self,
        path: &str,
        key: &str,
        value: T,
    ) -> Result<SrsClientResp, SrsClientError> {
        let response = self
            .http_client
            .get(
                self.base_url
                    .join(path)
                    .map_err(SrsClientError::IncorrectApiUrl)?,
            )
            .query(&[(key, value)])
            .send()
            .await
            .map_err(SrsClientError::RequestFailed)?;
        self.process_resp(response).await
    }
}
