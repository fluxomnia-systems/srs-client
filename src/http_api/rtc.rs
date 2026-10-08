//! SRS JSON signaling and WHIP/WHEP SDP session management.
use super::{SrsClient, SrsClientError};
use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RtcRequest {
    pub sdp: String,
    pub streamurl: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clientip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tid: Option<String>,
}

impl RtcRequest {
    #[must_use]
    pub fn new(streamurl: impl Into<String>, sdp: impl Into<String>) -> Self {
        Self {
            streamurl: streamurl.into(),
            sdp: sdp.into(),
            clientip: None,
            api: None,
            tid: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RtcResponse {
    pub code: i64,
    pub sdp: String,
    pub sessionid: String,
    #[serde(default)]
    pub server: String,
    #[serde(default)]
    pub service: String,
    #[serde(default)]
    pub pid: String,
}

/// SDP answer and the server-issued resource URL used to delete this session.
#[derive(Clone, Debug)]
pub struct SdpSession {
    pub sdp: String,
    pub location: Url,
    pub etag: Option<String>,
}

impl SrsClient {
    /// Publishes using SRS JSON signaling. Query options include codecs and candidate IP.
    ///
    /// # Errors
    /// Returns transport, HTTP, SRS application, or signaling-response errors.
    pub async fn rtc_publish(
        &self,
        offer: &RtcRequest,
        query: &[(&str, &str)],
    ) -> Result<RtcResponse, SrsClientError> {
        self.rtc_exchange("/rtc/v1/publish/", offer, query).await
    }

    /// Plays using SRS JSON signaling. Query options include codecs and candidate IP.
    ///
    /// # Errors
    /// Returns transport, HTTP, SRS application, or signaling-response errors.
    pub async fn rtc_play(
        &self,
        offer: &RtcRequest,
        query: &[(&str, &str)],
    ) -> Result<RtcResponse, SrsClientError> {
        self.rtc_exchange("/rtc/v1/play/", offer, query).await
    }

    async fn rtc_exchange(
        &self,
        path: &str,
        offer: &RtcRequest,
        query: &[(&str, &str)],
    ) -> Result<RtcResponse, SrsClientError> {
        let url = self
            .base_url
            .join(path)
            .map_err(SrsClientError::IncorrectApiUrl)?;
        let response = self
            .http_client
            .post(url)
            .query(query)
            .json(offer)
            .send()
            .await
            .map_err(SrsClientError::RequestFailed)?;
        serde_json::from_value(Self::json_response(response).await?)
            .map_err(SrsClientError::JsonDeserializeError)
    }

    /// Creates a WHIP publishing session. Include `app` and `stream` in query options.
    ///
    /// # Errors
    /// Returns errors for failed negotiation, invalid SDP responses, or unsafe resource URLs.
    pub async fn whip(
        &self,
        sdp: &str,
        query: &[(&str, &str)],
    ) -> Result<SdpSession, SrsClientError> {
        self.sdp_exchange("/rtc/v1/whip/", sdp, query).await
    }

    /// Creates a WHEP playback session. Include `app` and `stream` in query options.
    ///
    /// # Errors
    /// Returns errors for failed negotiation, invalid SDP responses, or unsafe resource URLs.
    pub async fn whep(
        &self,
        sdp: &str,
        query: &[(&str, &str)],
    ) -> Result<SdpSession, SrsClientError> {
        self.sdp_exchange("/rtc/v1/whep/", sdp, query).await
    }

    async fn sdp_exchange(
        &self,
        path: &str,
        sdp: &str,
        query: &[(&str, &str)],
    ) -> Result<SdpSession, SrsClientError> {
        let url = self
            .base_url
            .join(path)
            .map_err(SrsClientError::IncorrectApiUrl)?;
        let response = self
            .http_client
            .post(url)
            .query(query)
            .header(header::CONTENT_TYPE, "application/sdp")
            .body(sdp.to_owned())
            .send()
            .await
            .map_err(SrsClientError::RequestFailed)?;
        if response.status() != StatusCode::CREATED {
            if response.status().is_success() {
                return Err(SrsClientError::UnexpectedResponse("201 Created SDP answer"));
            }
            return Err(SrsClientError::BadStatus(response.status()));
        }
        if response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_none_or(|v| {
                !v.split(';')
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .eq_ignore_ascii_case("application/sdp")
            })
        {
            return Err(SrsClientError::UnexpectedResponse("application/sdp answer"));
        }
        let location = response
            .headers()
            .get(header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(SrsClientError::UnexpectedResponse(
                "session Location header",
            ))?;
        let location = response
            .url()
            .join(location)
            .map_err(SrsClientError::IncorrectApiUrl)?;
        self.validate_session_url(&location)?;
        let etag = response
            .headers()
            .get(header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let sdp = response
            .text()
            .await
            .map_err(SrsClientError::RequestFailed)?;
        if !sdp.starts_with("v=0") {
            return Err(SrsClientError::UnexpectedResponse("SDP answer"));
        }
        Ok(SdpSession {
            sdp,
            location,
            etag,
        })
    }

    fn validate_session_url(&self, location: &Url) -> Result<(), SrsClientError> {
        if location.origin() != self.base_url.origin()
            || !location.username().is_empty()
            || location.password().is_some()
            || !location.path().starts_with("/rtc/v1/")
        {
            return Err(SrsClientError::InvalidArgument(
                "session URL must belong to this SRS RTC API",
            ));
        }
        Ok(())
    }

    /// Deletes a WHIP/WHEP session using its server-issued Location (including token).
    ///
    /// # Errors
    /// Returns errors for foreign resource URLs, failed HTTP requests, or SRS errors.
    pub async fn delete_session(&self, session: &SdpSession) -> Result<(), SrsClientError> {
        self.validate_session_url(&session.location)?;
        let response = self
            .http_client
            .delete(session.location.clone())
            .send()
            .await
            .map_err(SrsClientError::RequestFailed)?;
        if !response.status().is_success() {
            return Err(SrsClientError::BadStatus(response.status()));
        }
        let body = response
            .text()
            .await
            .map_err(SrsClientError::RequestFailed)?;
        if !body.trim().is_empty() {
            let value: Value =
                serde_json::from_str(&body).map_err(SrsClientError::JsonDeserializeError)?;
            Self::check_code(&value)?;
        }
        Ok(())
    }

    /// Requests deliberate packet drops for the SRS RTC NACK diagnostic endpoint.
    ///
    /// # Errors
    /// Returns transport, HTTP, SRS application, or response-shape errors.
    pub async fn rtc_nack(&self, username: &str, drop: u32) -> Result<Value, SrsClientError> {
        let url = self
            .base_url
            .join("/rtc/v1/nack/")
            .map_err(SrsClientError::IncorrectApiUrl)?;
        let response = self
            .http_client
            .get(url)
            .query(&[("username", username), ("drop", &drop.to_string())])
            .send()
            .await
            .map_err(SrsClientError::RequestFailed)?;
        let value = Self::json_response(response).await?;
        if value.get("urls").is_some() {
            return Err(SrsClientError::UnsupportedEndpoint(
                "/rtc/v1/nack/".to_owned(),
            ));
        }
        if value.get("query").is_none() {
            return Err(SrsClientError::UnexpectedResponse("RTC NACK query"));
        }
        Ok(value)
    }
}
