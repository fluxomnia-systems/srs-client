---
lat:
  require-code-mention: true
---
# SRS Client Tests

These contracts verify the upgraded SRS client against malformed and valid HTTP responses, complete callbacks, and pinned live SRS releases.

## Application errors

Verify that nonzero SRS codes, unsuccessful HTTP status, missing codes, and malformed kickoff responses remain errors instead of reporting successful operations.

## Resource absence

Verify that only the correct resource-specific not-found code becomes None and that permission failures or empty success responses are not mistaken for absent resources.

## Response discrimination

Verify that unknown or incomplete data objects fail decoding and that the real request-debug payload becomes Requests rather than silently becoming Clusters.

## Management routes

Verify discovery, request diagnostics, RAW configuration, authors, reload initiation, and reload status use their actual endpoint paths and return the intended payloads.

## Optional features

Verify unavailable diagnostics reject discovery fallback, disabled metrics preserve SRS errors, and enabled diagnostic JSON/text responses decode correctly with the required query parameters.

## Callbacks

Verify all supported callback events round-trip their event-specific and future fields, close events allow an absent stream, and forwarding acknowledgements contain data.urls.

## Transport and parameters

Verify a reused configured transport sends authentication headers, query parameters are encoded, pagination is correct, and invalid IDs or pagination are rejected before requests.

## RTC JSON signaling

Verify publish/play send JSON offers to RTC paths, parse session answers, preserve SRS negotiation errors, and encode NACK diagnostic parameters.

## WHIP and WHEP lifecycle

Verify SDP content types, HTTP 201 handling, answer and ETag retention, and session deletion using the returned resource URL including its session token.

## Invalid SDP responses

Verify index fallback, missing Location, foreign session origins, and incorrect content types cannot be accepted as successful SDP negotiation.

## Live management

Verify actual pinned SRS servers provide correctly typed management data, missing-resource codes, enabled metrics/reload, and explicit errors for diagnostics absent from standard builds.

## Live RTC signaling

Verify real SRS servers negotiate WHIP/WHEP and JSON RTC sessions and accept deletion of negotiated WHIP/WHEP resources without requiring a completed media handshake.

## Live media resources

Publish synthetic H264/AAC using ffmpeg, verify live stream/client/vhost lookup and pagination, then kick off the publisher and confirm its client disappears.
