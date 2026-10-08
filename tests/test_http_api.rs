use srs_client::{SrsClientResp, SrsClientRespData};

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
                summary.urls.as_ref().expect("summary urls").streams,
                "manage all streams or specified stream"
            );
            assert_eq!(
                summary.tests.as_ref().expect("summary tests").requests,
                "ok"
            );
        }
        _ => panic!("expected summary response"),
    }

    Ok(())
}

#[test]
fn test_summaries_response() -> Result<(), Box<dyn std::error::Error>> {
    let response: SrsClientResp =
        serde_json::from_str(include_str!("fixtures/srs-summaries.json"))?;

    match response.data {
        SrsClientRespData::Summaries { data: summary } => {
            assert_eq!(summary.ok, Some(true));
            assert!(summary.now_ms.is_some());
            assert!(summary.self_.is_some());
            assert!(summary.system.is_some());
        }
        _ => panic!("expected runtime summaries response"),
    }

    Ok(())
}

#[test]
fn test_clusters_response() -> Result<(), Box<dyn std::error::Error>> {
    let response: SrsClientResp = serde_json::from_str(include_str!("fixtures/srs-clusters.json"))?;

    match response.data {
        SrsClientRespData::Clusters { data: clusters } => {
            let query = clusters.query.expect("clusters query");
            assert_eq!(query.ip, "");
            assert_eq!(query.vhost, "");
            assert_eq!(query.app, "");
            assert_eq!(query.stream, "");
            assert!(clusters.origin.is_none());
        }
        _ => panic!("expected clusters response"),
    }

    Ok(())
}
