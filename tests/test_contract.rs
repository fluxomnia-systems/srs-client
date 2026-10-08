mod support;
use serde_json::{json, Value};
use srs_client::*;
use support::{json as reply, response, server};

// @lat: [[tests#SRS Client Tests#Application errors]]
#[tokio::test]
async fn application_codes_and_http_failures_are_errors() {
    let (url, handle) = server(vec![
        reply(r#"{"code":2049}"#),
        reply(r#"{"code":1061,"message":"disabled"}"#),
        response("401 Unauthorized", "", ""),
        reply(r#"{"data":{}}"#),
        reply(r#"{"code":0,"unexpected":true}"#),
    ])
    .await;
    let client = SrsClient::build(url).unwrap();
    assert!(matches!(
        client.kickoff_client("missing").await,
        Err(SrsClientError::ApiError(2049))
    ));
    assert!(matches!(
        client.reload().await,
        Err(SrsClientError::ApiError(1061))
    ));
    assert!(matches!(
        client.get_version().await,
        Err(SrsClientError::BadStatus(reqwest::StatusCode::UNAUTHORIZED))
    ));
    assert!(matches!(
        client.get_version().await,
        Err(SrsClientError::UnexpectedResponse(_))
    ));
    assert!(matches!(
        client.kickoff_client("missing").await,
        Err(SrsClientError::JsonDeserializeError(_))
    ));
    assert_eq!(handle.await.unwrap().len(), 5);
}

// @lat: [[tests#SRS Client Tests#Resource absence]]
#[tokio::test]
async fn only_resource_specific_not_found_becomes_none() {
    let (url, handle) = server(vec![
        reply(r#"{"code":2014}"#),
        reply(r#"{"code":2048}"#),
        reply(r#"{"code":2049}"#),
        reply(r#"{"code":1061}"#),
        reply(r#"{"code":0}"#),
    ])
    .await;
    let client = SrsClient::build(url).unwrap();
    assert!(client.get_vhost_item("missing").await.unwrap().is_none());
    assert!(client.get_stream_item("missing").await.unwrap().is_none());
    assert!(client.get_client_item("missing").await.unwrap().is_none());
    assert!(matches!(
        client.get_client_item("missing").await,
        Err(SrsClientError::ApiError(1061))
    ));
    assert!(matches!(
        client.get_stream_item("missing").await,
        Err(SrsClientError::UnexpectedResponse(_))
    ));
    handle.await.unwrap();
}

// @lat: [[tests#SRS Client Tests#Response discrimination]]
#[test]
fn response_decoding_never_falls_back_to_clusters() {
    for data in [
        json!({"future":1}),
        json!({}),
        json!({"version":"8.0.48"}),
        json!({"query":{"page":"summary"}}),
    ] {
        assert!(serde_json::from_value::<SrsClientResp>(json!({"code":0,"data":data})).is_err());
    }
    let value = json!({"code":0,
            "data":{"uri":"http://srs/api/v1/tests/requests",
            "path":"/api/v1/tests/requests",
            "METHOD":"GET",
            "headers":{}}});
    assert!(matches!(
        serde_json::from_value::<SrsClientResp>(value).unwrap().data,
        SrsClientRespData::Requests { .. }
    ));
}

// @lat: [[tests#SRS Client Tests#Management routes]]
#[tokio::test]
async fn management_routes_return_their_actual_payloads() {
    let (url, handle) = server(vec![
        reply(include_str!("fixtures/srs-summary.json")),
        reply(
            r#"{
  "code": 0,
  "data": {
    "uri": "u",
    "path": "/api/v1/tests/requests",
    "METHOD": "GET",
    "headers": {}
  }
}"#,
        ),
        reply(r#"{"code":0,"http_api":{"enabled":true,"raw_api":{"enabled":true}}}"#),
        reply(r#"{"code":0,"data":{"license":"MIT","contributors":"SRS"}}"#),
        reply(r#"{"code":0}"#),
        reply(r#"{"code":0,"data":{"err":0,"msg":"","state":2,"rid":"reload-1"}}"#),
    ])
    .await;
    let client = SrsClient::build(url).unwrap();
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
    assert_eq!(
        client.get_raw_config().await.unwrap().http_api["enabled"],
        true
    );
    assert_eq!(client.get_authors().await.unwrap().license, "MIT");
    client.reload().await.unwrap();
    assert_eq!(client.get_reload_status().await.unwrap().rid, "reload-1");
    let requests = handle.await.unwrap();
    for (request, path) in requests.iter().zip([
        "/api/v1/",
        "/api/v1/tests/requests",
        "/api/v1/raw?rpc=raw",
        "/api/v1/authors",
        "/api/v1/raw?rpc=reload",
        "/api/v1/raw?rpc=reload-fetch",
    ]) {
        assert!(
            request.starts_with(&format!("GET {path} HTTP/1.1")),
            "{request}"
        );
    }
}

// @lat: [[tests#SRS Client Tests#Optional features]]
#[tokio::test]
async fn optional_features_reject_discovery_fallback_and_decode_real_data() {
    let index = include_str!("fixtures/srs-summary.json");
    let (url, handle) = server(vec![
        reply(index),
        reply(index),
        reply(index),
        reply(index),
        reply(r#"{"code":3000}"#),
        reply(
            r#"{
  "code": 0,
  "data": {
    "query": {},
    "release_rate": 1.5,
    "generic": {
      "heap_size": 100
    },
    "tcmalloc": {
      "pageheap_free_bytes": 10
    }
  }
}"#,
        ),
        reply(r#"{"code":0,"data":{"signal":"SIGUSR1","signo":10,"help":"help"}}"#),
        reply(
            r#"{
  "code": 0,
  "data": {
    "check": "quick",
    "call": "VALGRIND_DO_QUICK_LEAK_CHECK",
    "help": "help",
    "see": "url"
  }
}"#,
        ),
        response(
            "200 OK",
            "Content-Type: text/plain\r\n",
            "# HELP srs_build_info SRS\n# TYPE srs_build_info gauge\nsrs_build_info 1\n",
        ),
        response("200 OK", "", "MALLOC: 100 bytes in use\n"),
    ])
    .await;
    let client = SrsClient::build(url).unwrap();
    assert!(matches!(
        client.get_tcmalloc().await,
        Err(SrsClientError::UnsupportedEndpoint(_))
    ));
    assert!(matches!(
        client.get_tcmalloc_stats().await,
        Err(SrsClientError::UnsupportedEndpoint(_))
    ));
    assert!(matches!(
        client.signal(SrsSignal::ReopenLog).await,
        Err(SrsClientError::UnsupportedEndpoint(_))
    ));
    assert!(matches!(
        client.valgrind(ValgrindCheck::Quick).await,
        Err(SrsClientError::UnsupportedEndpoint(_))
    ));
    assert!(matches!(
        client.get_metrics().await,
        Err(SrsClientError::ApiError(3000))
    ));
    assert!(matches!(
        client.get_tcmalloc().await.unwrap().data,
        SrsClientRespData::Tcmalloc { .. }
    ));
    assert_eq!(
        client.signal(SrsSignal::ReopenLog).await.unwrap().signal,
        "SIGUSR1"
    );
    assert_eq!(
        client.valgrind(ValgrindCheck::Quick).await.unwrap().check,
        "quick"
    );
    assert!(client
        .get_metrics()
        .await
        .unwrap()
        .contains("srs_build_info 1"));
    assert!(client
        .get_tcmalloc_stats()
        .await
        .unwrap()
        .contains("100 bytes"));
    let requests = handle.await.unwrap();
    assert!(requests[6].starts_with("GET /api/v1/signal?signo=SIGUSR1 "));
    assert!(requests[7].starts_with("GET /api/v1/valgrind?check=quick "));
}

// @lat: [[tests#SRS Client Tests#Callbacks]]
#[test]
fn callbacks_preserve_event_specific_and_future_fields() {
    for action in [
        "on_connect",
        "on_close",
        "on_publish",
        "on_unpublish",
        "on_play",
        "on_stop",
        "on_dvr",
        "on_hls",
        "on_forward",
    ] {
        let payload = json!({"server_id":"server",
            "service_id":"service",
            "action":action,
            "client_id":"client",
            "ip":"::1",
            "vhost":"__defaultVhost__",
            "app":"live",
            "stream":"test",
            "param":"?token=abc",
            "tcUrl":"rtmp://srs/live",
            "pageUrl":"https://example.org",
            "stream_url":"live/test",
            "stream_id":"vid-1",
            "send_bytes":10,
            "recv_bytes":20,
            "cwd":"/srs",
            "file":"test.ts",
            "duration":1.5,
            "url":"test.ts",
            "m3u8":"test.m3u8",
            "m3u8_url":"test.m3u8",
            "seq_no":3,
            "future":{"keep":true}});
        let callback: SrsCallbackReq = serde_json::from_value(payload.clone()).unwrap();
        assert_eq!(callback.app_stream(), "live/test");
        assert_eq!(serde_json::to_value(callback).unwrap(), payload);
    }
    let close: SrsCallbackReq = serde_json::from_value(json!({"server_id":"s",
            "action":"on_close",
            "client_id":"c",
            "ip":"127.0.0.1",
            "vhost":"v",
            "app":"live"}))
    .unwrap();
    assert!(close.stream.is_none());
    let forward = SrsForwardResponse {
        code: 0,
        data: SrsForwardData {
            urls: vec!["rtmp://other/live/test".into()],
        },
    };
    assert_eq!(
        serde_json::to_value(forward).unwrap(),
        json!({"code":0,"data":{"urls":["rtmp://other/live/test"]}})
    );
}

// @lat: [[tests#SRS Client Tests#Transport and parameters]]
#[tokio::test]
async fn reusable_transport_preserves_auth_and_encodes_queries() {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        "Basic dXNlcjpwYXNz".parse().unwrap(),
    );
    let http = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();
    let (url, handle) = server(vec![
        reply(include_str!("fixtures/srs-clusters.json")),
        reply(r#"{"code":0,"streams":[]}"#),
    ])
    .await;
    let client = SrsClient::with_http_client(url, http).unwrap();
    client
        .query_clusters(&ClusterQuery {
            app: "live & test".into(),
            stream: "a/b?token=x".into(),
            ..ClusterQuery::default()
        })
        .await
        .unwrap();
    assert!(client.get_stream_page_list(3, 20).await.unwrap().is_empty());
    assert!(matches!(
        client.get_stream_item("../clients").await,
        Err(SrsClientError::InvalidArgument(_))
    ));
    assert!(matches!(
        client.get_clients_page(-1, 10).await,
        Err(SrsClientError::InvalidArgument(_))
    ));
    let requests = handle.await.unwrap();
    assert!(requests[0].contains("app=live+%26+test"));
    assert!(requests[0].contains("stream=a%2Fb%3Ftoken%3Dx"));
    assert!(requests.iter().all(|r| r
        .to_ascii_lowercase()
        .contains("authorization: basic dxnlcjpwyxnz")));
    assert!(requests[1].starts_with("GET /api/v1/streams?start=3&count=20 "));
}

// @lat: [[tests#SRS Client Tests#RTC JSON signaling]]
#[tokio::test]
async fn rtc_json_signaling_posts_offers_and_handles_errors() {
    let answer = r#"{"code":0,"sdp":"v=0\r\n","sessionid":"session"}"#;
    let (url, handle) = server(vec![
        reply(answer),
        reply(answer),
        reply(r#"{"code":5018,"message":"bad SDP"}"#),
        reply(r#"{"code":0,"query":{"username":"session","drop":"1"}}"#),
    ])
    .await;
    let client = SrsClient::build(url).unwrap();
    let offer = RtcRequest::new("webrtc://srs/live/test", "v=0\r\n");
    assert_eq!(
        client
            .rtc_publish(&offer, &[("candidate", "127.0.0.1")])
            .await
            .unwrap()
            .sessionid,
        "session"
    );
    assert_eq!(client.rtc_play(&offer, &[]).await.unwrap().sdp, "v=0\r\n");
    assert!(matches!(
        client.rtc_publish(&offer, &[]).await,
        Err(SrsClientError::ApiError(5018))
    ));
    client.rtc_nack("session", 1).await.unwrap();
    let requests = handle.await.unwrap();
    assert!(requests[0].starts_with("POST /rtc/v1/publish/?candidate=127.0.0.1 "));
    let body: Value = serde_json::from_str(requests[0].split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(body["streamurl"], "webrtc://srs/live/test");
    assert!(requests[1].starts_with("POST /rtc/v1/play/ "));
    assert!(requests[3].starts_with("GET /rtc/v1/nack/?username=session&drop=1 "));
}

// @lat: [[tests#SRS Client Tests#WHIP and WHEP lifecycle]]
#[tokio::test]
async fn sdp_sessions_use_location_and_delete_token() {
    let answer = response(
        "201 Created",
        concat!(
            "Content-Type: application/sdp\r\n",
            "Location: /rtc/v1/whip/?action=delete&session=s&token=t\r\n",
            "ETag: session-1\r\n"
        ),
        "v=0\r\n",
    );
    let (url, handle) = server(vec![
        answer.clone(),
        response("200 OK", "", ""),
        answer,
        response("204 No Content", "", ""),
    ])
    .await;
    let client = SrsClient::build(url).unwrap();
    let query = [("app", "live"), ("stream", "a & b")];
    let session = client.whip("v=0\r\n", &query).await.unwrap();
    assert_eq!(session.etag.as_deref(), Some("session-1"));
    client.delete_session(&session).await.unwrap();
    let session = client.whep("v=0\r\n", &query).await.unwrap();
    client.delete_session(&session).await.unwrap();
    let requests = handle.await.unwrap();
    assert!(requests[0].starts_with("POST /rtc/v1/whip/?app=live&stream=a+%26+b "));
    assert!(requests[0].contains("content-type: application/sdp"));
    assert!(requests[1].starts_with("DELETE /rtc/v1/whip/?action=delete&session=s&token=t "));
    assert!(requests[2].starts_with("POST /rtc/v1/whep/?"));
}

// @lat: [[tests#SRS Client Tests#Invalid SDP responses]]
#[tokio::test]
async fn sdp_fallback_and_foreign_session_locations_are_rejected() {
    let (url, handle) = server(vec![
        reply(r#"{"code":0,"urls":{}}"#),
        response(
            "201 Created",
            "Content-Type: application/sdp\r\nLocation: https://foreign.example/rtc/v1/whip/\r\n",
            "v=0\r\n",
        ),
        response(
            "201 Created",
            "Content-Type: application/sdp\r\n",
            "v=0\r\n",
        ),
        response(
            "201 Created",
            "Content-Type: text/html\r\nLocation: /rtc/v1/whip/\r\n",
            "<html>oops</html>",
        ),
    ])
    .await;
    let client = SrsClient::build(url).unwrap();
    assert!(matches!(
        client.whip("offer", &[]).await,
        Err(SrsClientError::UnexpectedResponse(_))
    ));
    assert!(matches!(
        client.whip("offer", &[]).await,
        Err(SrsClientError::InvalidArgument(_))
    ));
    assert!(matches!(
        client.whep("offer", &[]).await,
        Err(SrsClientError::UnexpectedResponse(_))
    ));
    assert!(matches!(
        client.whep("offer", &[]).await,
        Err(SrsClientError::UnexpectedResponse(_))
    ));
    handle.await.unwrap();
}
