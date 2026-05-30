use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::{OpenApi, ToSchema};

/// This service's identity. `srvcs-digitsum` is a leaf: it depends on no other
/// service and does all of its work as local `i64` arithmetic.
pub const SERVICE: &str = "srvcs-digitsum";
pub const CONCERN: &str = "number theory: sum of decimal digits";
pub const DEPENDS_ON: &[&str] = &[];

#[derive(Serialize, ToSchema)]
pub struct Info {
    pub service: &'static str,
    pub concern: &'static str,
    pub depends_on: Vec<&'static str>,
}

/// `GET /` — service identity (srvcs service standard).
#[utoipa::path(get, path = "/", responses((status = 200, body = Info)))]
pub async fn index() -> Json<Info> {
    Json(Info {
        service: SERVICE,
        concern: CONCERN,
        depends_on: DEPENDS_ON.to_vec(),
    })
}

#[derive(Deserialize, ToSchema)]
pub struct EvalRequest {
    /// The integer whose decimal digits are summed.
    #[schema(value_type = Object)]
    pub value: Value,
}

#[derive(Serialize, ToSchema)]
pub struct DigitSumResponse {
    #[schema(value_type = Object)]
    pub value: Value,
    pub result: i64,
}

/// Coerce a numeric JSON value to an integer, accepting whole floats (`123.0`)
/// but rejecting genuinely fractional ones (`12.3`).
fn as_integer(value: &Value) -> Option<i64> {
    value.as_i64().or_else(|| {
        value
            .as_f64()
            .filter(|f| f.fract() == 0.0)
            .map(|f| f as i64)
    })
}

/// The single concern: the sum of the decimal digits of `n`.
///
/// The sign is ignored — `digit_sum(-49) == digit_sum(49) == 13`. Computed
/// locally by repeatedly peeling off the least-significant digit.
pub fn digit_sum(n: i64) -> i64 {
    let mut m = n.unsigned_abs();
    let mut sum: i64 = 0;
    loop {
        sum += (m % 10) as i64;
        m /= 10;
        if m == 0 {
            break;
        }
    }
    sum
}

fn ok(value: Value, result: i64) -> Response {
    (
        StatusCode::OK,
        Json(json!({ "value": value, "result": result })),
    )
        .into_response()
}

fn invalid(reason: &str) -> Response {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(json!({ "error": reason })),
    )
        .into_response()
}

/// `POST /` — the sum of the decimal digits of `value`.
///
/// This is a leaf: it does no I/O. The value is read as an `i64`, its sign is
/// dropped, and its decimal digits are summed locally. Non-integer input is
/// rejected with `422`.
#[utoipa::path(
    post,
    path = "/",
    request_body = EvalRequest,
    responses(
        (status = 200, body = DigitSumResponse),
        (status = 422, description = "value is not an integer"),
        (status = 500, description = "internal error")
    )
)]
pub async fn evaluate(Json(req): Json<EvalRequest>) -> Response {
    let Some(n) = as_integer(&req.value) else {
        return invalid("value is not an integer");
    };
    ok(req.value, digit_sum(n))
}

#[derive(OpenApi)]
#[openapi(
    paths(index, evaluate),
    components(schemas(Info, EvalRequest, DigitSumResponse))
)]
pub struct ApiDoc;

/// Serve OpenAPI document
pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_documents_routes() {
        let doc = ApiDoc::openapi();
        let root = doc.paths.paths.get("/").expect("path / present");
        assert!(root.get.is_some(), "GET / documented");
        assert!(root.post.is_some(), "POST / documented");
    }

    #[test]
    fn asserted_cases_from_the_algorithm() {
        assert_eq!(digit_sum(123), 6);
        assert_eq!(digit_sum(-49), 13);
        assert_eq!(digit_sum(0), 0);
    }

    #[test]
    fn sign_is_ignored() {
        assert_eq!(digit_sum(49), digit_sum(-49));
        assert_eq!(digit_sum(1000), digit_sum(-1000));
    }

    #[test]
    fn single_digits_sum_to_themselves() {
        for d in 0..=9 {
            assert_eq!(digit_sum(d), d);
        }
    }

    #[test]
    fn trailing_zeros_contribute_nothing() {
        assert_eq!(digit_sum(100), 1);
        assert_eq!(digit_sum(205), 7);
        assert_eq!(digit_sum(9_000_000), 9);
    }

    #[test]
    fn handles_i64_extremes() {
        // i64::MAX = 9223372036854775807 -> digits sum to 88.
        assert_eq!(digit_sum(i64::MAX), 88);
        // i64::MIN's magnitude (9223372036854775808) is representable via
        // unsigned_abs, so this must not panic.
        assert_eq!(digit_sum(i64::MIN), 89);
    }

    #[test]
    fn whole_floats_are_integers_but_fractions_are_not() {
        assert_eq!(as_integer(&json!(123)), Some(123));
        assert_eq!(as_integer(&json!(123.0)), Some(123));
        assert_eq!(as_integer(&json!(-49.0)), Some(-49));
        assert_eq!(as_integer(&json!(12.3)), None);
        assert_eq!(as_integer(&json!("123")), None);
    }

    #[tokio::test]
    async fn index_reports_identity() {
        let Json(info) = index().await;
        assert_eq!(info.service, "srvcs-digitsum");
        assert_eq!(info.concern, "number theory: sum of decimal digits");
        assert!(info.depends_on.is_empty());
    }
}
