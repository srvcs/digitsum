use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use srvcs_digitsum::{health, router, telemetry};
use tower::ServiceExt;

async fn status_of(uri: &str) -> StatusCode {
    let app = router(telemetry::metrics_handle_for_tests());
    app.oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

/// POST a JSON body to `/` and return (status, parsed JSON response).
async fn post_eval(body: Value) -> (StatusCode, Value) {
    let app = router(telemetry::metrics_handle_for_tests());
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

#[tokio::test]
async fn index_ok() {
    assert_eq!(status_of("/").await, StatusCode::OK);
}

#[tokio::test]
async fn healthz_ok() {
    assert_eq!(status_of("/healthz").await, StatusCode::OK);
}

#[tokio::test]
async fn readyz_reflects_state() {
    health::set_ready(true);
    assert_eq!(status_of("/readyz").await, StatusCode::OK);
}

#[tokio::test]
async fn metrics_ok() {
    assert_eq!(status_of("/metrics").await, StatusCode::OK);
}

#[tokio::test]
async fn openapi_ok() {
    assert_eq!(status_of("/openapi.json").await, StatusCode::OK);
}

#[tokio::test]
async fn index_reports_identity() {
    let app = router(telemetry::metrics_handle_for_tests());
    let res = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["service"], "srvcs-digitsum");
    assert_eq!(body["concern"], "number theory: sum of decimal digits");
    assert_eq!(body["depends_on"], json!([]));
}

#[tokio::test]
async fn digitsum_of_123_is_6() {
    let (status, body) = post_eval(json!({ "value": 123 })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["value"], json!(123));
    assert_eq!(body["result"], json!(6));
}

#[tokio::test]
async fn digitsum_of_negative_49_is_13() {
    let (status, body) = post_eval(json!({ "value": -49 })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["value"], json!(-49));
    assert_eq!(body["result"], json!(13));
}

#[tokio::test]
async fn digitsum_of_zero_is_0() {
    let (status, body) = post_eval(json!({ "value": 0 })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["value"], json!(0));
    assert_eq!(body["result"], json!(0));
}

#[tokio::test]
async fn whole_float_is_accepted() {
    let (status, body) = post_eval(json!({ "value": 123.0 })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"], json!(6));
}

#[tokio::test]
async fn fractional_value_is_rejected() {
    let (status, _) = post_eval(json!({ "value": 12.3 })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn non_numeric_value_is_rejected() {
    let (status, _) = post_eval(json!({ "value": "123" })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn missing_value_field_is_rejected() {
    // Missing the `value` field is a client error, not a 500.
    let (status, _) = post_eval(json!({ "notvalue": 1 })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn generates_request_id_when_absent() {
    let app = router(telemetry::metrics_handle_for_tests());
    let res = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        res.headers().contains_key("x-request-id"),
        "response must carry a generated x-request-id"
    );
}
