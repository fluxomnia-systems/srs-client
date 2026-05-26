use srs_client::{SrsClient, SrsClientError, SrsClientResp, SrsClientRespData};
use std::env;
use tokio;
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::TcpListener,
};

// #[tokio::test]
// async fn test_kickoff_client() -> Result<(), Box<dyn std::error::Error>> {
//     let srs_http_api_url =
//         env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
//     let client = SrsClient::build(&srs_http_api_url)?;
//     let result: Result<SrsClientResp, SrsClientError> =
//         client.kickoff_client("21233").await;
//     assert!(result.is_ok());
//     Ok(())
// }

#[tokio::test]
async fn test_kickoff_client_accepts_minimal_success_response(
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept request");
        let mut buffer = [0_u8; 1024];
        let read = socket.read(&mut buffer).await.expect("read request");
        let request = String::from_utf8_lossy(&buffer[..read]);
        assert!(request.starts_with("DELETE /api/v1/clients/client-1/ "));
        socket
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 10\r\n\r\n{\"code\":0}",
            )
            .await
            .expect("write response");
    });

    let client = SrsClient::build(format!("http://{addr}"))?;
    let response = client.kickoff_client("client-1").await?;
    server.await?;

    assert_eq!(response.code, 0);
    assert!(matches!(response.data, SrsClientRespData::Empty));
    Ok(())
}

#[tokio::test]
async fn test_kickoff_client_rejects_unexpected_minimal_response_shape(
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept request");
        let mut buffer = [0_u8; 1024];
        let read = socket.read(&mut buffer).await.expect("read request");
        let request = String::from_utf8_lossy(&buffer[..read]);
        assert!(request.starts_with("DELETE /api/v1/clients/client-1/ "));
        let body = r#"{"code":0,"unexpected":"value"}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        socket
            .write_all(response.as_bytes())
            .await
            .expect("write response");
    });

    let client = SrsClient::build(format!("http://{addr}"))?;
    let result = client.kickoff_client("client-1").await;
    server.await?;

    assert!(matches!(
        result,
        Err(SrsClientError::JsonDeserializeError(_))
    ));
    Ok(())
}

#[tokio::test]
async fn test_get_version() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url = env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> = client.get_version().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_get_vhosts() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url = env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> = client.get_vhosts().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_get_streams() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url = env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> = client.get_streams().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_get_clients() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url = env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> = client.get_clients().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_get_features() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url = env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> = client.get_features().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_get_rusages() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url = env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> = client.get_rusages().await;
    assert!(result.is_ok());
    Ok(())
}
#[tokio::test]
async fn test_get_self_proc_stats() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url = env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> = client.get_self_proc_stats().await;
    assert!(result.is_ok());
    Ok(())
}
#[tokio::test]
async fn test_get_system_proc_stats() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url = env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> = client.get_system_proc_stats().await;
    assert!(result.is_ok());
    Ok(())
}
#[tokio::test]
async fn test_get_meminfos() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url = env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> = client.get_meminfos().await;
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_active_stream_response_with_media_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let response: SrsClientResp = serde_json::from_str(include_str!("fixtures/srs-streams.json"))?;

    match response.data {
        SrsClientRespData::Streams { streams } => {
            assert_eq!(streams.len(), 2);

            let video = streams[0].video.as_ref().expect("video metadata");
            assert_eq!(video.codec, "H264");
            assert_eq!(video.profile, "High");
            assert_eq!(video.level, "3.2");
            assert_eq!(video.width, 1280);
            assert_eq!(video.height, 720);

            let audio = streams[0].audio.as_ref().expect("audio metadata");
            assert_eq!(audio.codec, "AAC");
            assert_eq!(audio.sample_rate, 44100);
            assert_eq!(audio.channel, 2);
            assert_eq!(audio.profile, "LC");
        }
        _ => panic!("expected streams response"),
    }

    Ok(())
}

#[test]
fn test_single_resource_responses() -> Result<(), Box<dyn std::error::Error>> {
    let response: SrsClientResp = serde_json::from_str(include_str!("fixtures/srs-stream.json"))?;
    match response.data {
        SrsClientRespData::Stream { stream } => {
            assert_eq!(stream.id, "vid-c99a4wx");
            assert_eq!(stream.publish.cid.as_deref(), Some("nf14l8c0"));
        }
        _ => panic!("expected stream response"),
    }

    let response: SrsClientResp = serde_json::from_str(include_str!("fixtures/srs-client.json"))?;
    match response.data {
        SrsClientRespData::Client { client } => {
            assert_eq!(client.id, "206sj057");
            assert_eq!(client.r#type, "fmle-publish");
        }
        _ => panic!("expected client response"),
    }

    let response: SrsClientResp = serde_json::from_str(include_str!("fixtures/srs-vhost.json"))?;
    match response.data {
        SrsClientRespData::Vhost { vhost } => {
            assert_eq!(vhost.id, "vid-ibe77pd");
            assert_eq!(vhost.hls.fragment, Some(1.0));
        }
        _ => panic!("expected vhost response"),
    }

    Ok(())
}

#[test]
fn test_vhost_list_response() -> Result<(), Box<dyn std::error::Error>> {
    let response: SrsClientResp = serde_json::from_str(include_str!("fixtures/srs-vhosts.json"))?;

    match response.data {
        SrsClientRespData::Vhosts { vhosts } => {
            assert_eq!(vhosts.len(), 1);
            assert_eq!(vhosts[0].hls.fragment, Some(1.0));
        }
        _ => panic!("expected vhosts response"),
    }

    Ok(())
}

#[test]
fn test_summary_response() -> Result<(), Box<dyn std::error::Error>> {
    let response: SrsClientResp = serde_json::from_str(include_str!("fixtures/srs-summary.json"))?;

    match response.data {
        SrsClientRespData::Summary(summary) => {
            assert_eq!(
                summary.urls.streams,
                "manage all streams or specified stream"
            );
            assert_eq!(summary.tests.requests, "ok");
        }
        _ => panic!("expected summary response"),
    }

    Ok(())
}

#[test]
fn test_requests_and_configs_responses() -> Result<(), Box<dyn std::error::Error>> {
    let response: SrsClientResp = serde_json::from_str(include_str!("fixtures/srs-requests.json"))?;
    match response.data {
        SrsClientRespData::Summary(summary) => {
            assert_eq!(summary.urls.requests, "the request itself, for http debug");
            assert_eq!(summary.tests.requests, "show the request info");
            assert_eq!(
                summary.tests.vhost,
                "http vhost for http://error.srs.com:1985/api/v1/tests/errors"
            );
        }
        _ => panic!("expected requests summary response"),
    }

    let response: SrsClientResp = serde_json::from_str(include_str!("fixtures/srs-configs.json"))?;
    match response.data {
        SrsClientRespData::Summary(summary) => {
            assert_eq!(
                summary.urls.raw,
                "raw api for srs, support CUID srs for instance the config"
            );
            assert_eq!(
                summary.tests.redirects,
                "always redirect to /api/v1/test/errors"
            );
        }
        _ => panic!("expected configs summary response"),
    }

    Ok(())
}
