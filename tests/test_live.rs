//! Opt-in tests against pinned SRS images. Run with `-- --ignored`.
use srs_client::*;
use std::{env, process::Stdio, time::Duration};

fn client() -> SrsClient {
    SrsClient::build(env::var("SRS_HTTP_API_URL").expect("set SRS_HTTP_API_URL")).unwrap()
}

// @lat: [[tests#SRS Client Tests#Live management]]
#[tokio::test]
#[ignore = "requires SRS_HTTP_API_URL and a running SRS server"]
async fn live_management_contract() {
    let client = client();
    match client.get_version().await.unwrap().data {
        SrsClientRespData::Version { data } => {
            assert!(data.major >= 6);
            if let Ok(expected) = env::var("SRS_EXPECTED_VERSION") {
                assert_eq!(data.version, expected);
            }
        }
        other => panic!("wrong version response: {other:?}"),
    }
    assert!(client
        .get_summary()
        .await
        .unwrap()
        .unwrap()
        .now_ms
        .is_some());
    assert!(client
        .get_api_summary()
        .await
        .unwrap()
        .unwrap()
        .urls
        .is_some());
    assert_eq!(
        client.get_request_info().await.unwrap().path,
        "/api/v1/tests/requests"
    );
    assert_ne!(client.get_authors().await.unwrap().license, "");
    assert!(client.get_raw_config().await.unwrap().http_api.is_object());
    assert!(matches!(
        client.get_rusages().await.unwrap().data,
        SrsClientRespData::Rusages { .. }
    ));
    assert!(matches!(
        client.get_self_proc_stats().await.unwrap().data,
        SrsClientRespData::SelfProcStats { .. }
    ));
    assert!(matches!(
        client.get_system_proc_stats().await.unwrap().data,
        SrsClientRespData::SystemProcStats { .. }
    ));
    assert!(matches!(
        client.get_meminfos().await.unwrap().data,
        SrsClientRespData::MemInfos { .. }
    ));
    assert!(matches!(
        client.get_features().await.unwrap().data,
        SrsClientRespData::Feature { .. }
    ));
    let query = ClusterQuery {
        app: "live-check".into(),
        stream: "stream-check".into(),
        ..ClusterQuery::default()
    };
    assert_eq!(
        client
            .query_clusters(&query)
            .await
            .unwrap()
            .query
            .unwrap()
            .app,
        query.app
    );
    assert!(client.get_vhost_item("missing").await.unwrap().is_none());
    assert!(client.get_stream_item("missing").await.unwrap().is_none());
    assert!(client.get_client_item("missing").await.unwrap().is_none());
    assert!(matches!(
        client.kickoff_client("missing").await,
        Err(SrsClientError::ApiError(2049))
    ));
    // Standard images do not compile in tcmalloc, signal, or Valgrind handlers.
    assert!(matches!(
        client.get_tcmalloc().await,
        Err(SrsClientError::UnsupportedEndpoint(_))
    ));
    assert!(matches!(
        client.valgrind(ValgrindCheck::Quick).await,
        Err(SrsClientError::UnsupportedEndpoint(_))
    ));
    assert!(matches!(
        client.signal(SrsSignal::ReopenLog).await,
        Err(SrsClientError::UnsupportedEndpoint(_))
    ));
    // Test configuration enables the exporter and RAW reload.
    assert!(client.get_metrics().await.unwrap().contains("# HELP"));
    client.reload().await.unwrap();
    assert_ne!(client.get_reload_status().await.unwrap().rid, "");
}

// @lat: [[tests#SRS Client Tests#Live RTC signaling]]
#[tokio::test]
#[ignore = "requires a running SRS server with RTC enabled"]
async fn live_rtc_signaling_and_session_deletion() {
    let client = client();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let whip_stream = format!("compat-whip-{nonce}");
    let json_stream = format!("webrtc://localhost/live/compat-json-{nonce}");
    let offer = include_str!("fixtures/offer.sdp").replace('\n', "\r\n");
    let session = client
        .whip(&offer, &[("app", "live"), ("stream", &whip_stream)])
        .await
        .unwrap();
    assert!(session.sdp.contains("a=ice-ufrag:"));
    assert!(session.location.query().unwrap().contains("token="));
    let play_offer = offer.replace("a=sendonly", "a=recvonly");
    let player = client
        .whep(&play_offer, &[("app", "live"), ("stream", &whip_stream)])
        .await
        .unwrap();
    client.delete_session(&player).await.unwrap();
    client.delete_session(&session).await.unwrap();
    let publisher = client
        .rtc_publish(&RtcRequest::new(&json_stream, &offer), &[])
        .await
        .unwrap();
    assert_ne!(publisher.sessionid, "");
    let player = client
        .rtc_play(&RtcRequest::new(&json_stream, &play_offer), &[])
        .await
        .unwrap();
    assert_ne!(player.sessionid, "");
    assert!(matches!(
        client.rtc_nack(&publisher.sessionid, 1).await,
        Err(SrsClientError::UnsupportedEndpoint(_))
    ));
    // JSON signaling has no DELETE Location; these unconnected sessions expire in SRS.
}

// @lat: [[tests#SRS Client Tests#Live media resources]]
#[tokio::test]
#[ignore = "requires ffmpeg and SRS_RTMP_URL pointing to an isolated SRS server"]
async fn live_publish_pagination_and_kickoff() {
    let rtmp = env::var("SRS_RTMP_URL").expect("set SRS_RTMP_URL, e.g. rtmp://127.0.0.1:1935/live");
    let mut publisher = tokio::process::Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            "error",
            "-re",
            "-f",
            "lavfi",
            "-i",
            "color=c=black:s=160x90:r=5",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=1000:sample_rate=44100",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-tune",
            "zerolatency",
            "-c:a",
            "aac",
            "-f",
            "flv",
        ])
        .arg(format!("{}/compat-media", rtmp.trim_end_matches('/')))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let client = client();
    let stream = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let streams = client.get_stream_page_list(0, 100).await.unwrap();
            if let Some(stream) = streams
                .into_iter()
                .find(|s| s.name == "compat-media" && s.video.is_some() && s.audio.is_some())
            {
                break stream;
            }
            assert!(
                publisher.try_wait().unwrap().is_none(),
                "ffmpeg exited before publishing"
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("active stream with media metadata");
    assert_eq!(stream.video.as_ref().unwrap().codec, "H264");
    assert_eq!(stream.audio.as_ref().unwrap().codec, "AAC");
    assert!(client.get_stream_item(&stream.id).await.unwrap().is_some());
    assert!(client
        .get_vhost_item(&stream.vhost)
        .await
        .unwrap()
        .is_some());
    let cid = stream.publish.cid.unwrap();
    assert!(client
        .get_client_page_list(0, 100)
        .await
        .unwrap()
        .iter()
        .any(|c| c.id == cid));
    assert!(client.get_client_item(&cid).await.unwrap().unwrap().publish);
    client.kickoff_client(&cid).await.unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
        while client.get_client_item(&cid).await.unwrap().is_some() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("client removed after kickoff");
    let _ = publisher.kill().await;
    let _ = publisher.wait().await;
}
