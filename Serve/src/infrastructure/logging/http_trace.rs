use axum::http::{HeaderMap, Request, Response};
use std::time::Duration;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{
    DefaultOnBodyChunk, DefaultOnEos, MakeSpan, OnFailure, OnRequest, OnResponse, TraceLayer,
};
use tracing::{Span, error, info, warn};

/// Header name used for distributed tracing and request tracking
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Creates the request ID layer for generating a UUID for every incoming request
pub fn create_request_id_layer() -> SetRequestIdLayer<MakeRequestUuid> {
    SetRequestIdLayer::x_request_id(MakeRequestUuid)
}

/// Creates the request ID propagation layer so that response headers include the same x-request-id
pub fn create_propagate_request_id_layer() -> PropagateRequestIdLayer {
    PropagateRequestIdLayer::x_request_id()
}

/// Custom span builder that extracts method, uri, version, request ID, and client IP
#[derive(Clone, Debug)]
pub struct CustomMakeSpan;

impl<B> MakeSpan<B> for CustomMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let request_id = request
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");

        let client_ip = extract_client_ip(request.headers());

        tracing::info_span!(
            "http_request",
            method = %request.method(),
            uri = %request.uri().path(),
            query = request.uri().query().unwrap_or(""),
            request_id = %request_id,
            client_ip = client_ip.as_deref().unwrap_or("unknown"),
        )
    }
}

/// Custom on_request hook logging the start of an HTTP request
#[derive(Clone, Debug)]
pub struct CustomOnRequest;

impl<B> OnRequest<B> for CustomOnRequest {
    fn on_request(&mut self, request: &Request<B>, _span: &Span) {
        let req_id = request
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-");

        info!(
            target: "serve::http",
            request_id = %req_id,
            method = %request.method(),
            path = %request.uri().path(),
            "Incoming HTTP request"
        );
    }
}

/// Custom on_response hook logging latency and status code with level differentiation
#[derive(Clone, Debug)]
pub struct CustomOnResponse;

impl<B> OnResponse<B> for CustomOnResponse {
    fn on_response(self, response: &Response<B>, latency: Duration, _span: &Span) {
        let status = response.status();
        let latency_ms = latency.as_secs_f64() * 1000.0;

        let req_id = response
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-");

        if status.is_server_error() {
            error!(
                target: "serve::http",
                request_id = %req_id,
                status = %status.as_u16(),
                latency_ms = format_args!("{:.2}ms", latency_ms),
                "HTTP request completed with server error"
            );
        } else if status.is_client_error() {
            warn!(
                target: "serve::http",
                request_id = %req_id,
                status = %status.as_u16(),
                latency_ms = format_args!("{:.2}ms", latency_ms),
                "⚠HTTP request completed with client error"
            );
        } else {
            info!(
                target: "serve::http",
                request_id = %req_id,
                status = %status.as_u16(),
                latency_ms = format_args!("{:.2}ms", latency_ms),
                "HTTP request completed successfully"
            );
        }
    }
}

/// Custom on_failure hook logging abrupt failures
#[derive(Clone, Debug)]
pub struct CustomOnFailure;

impl OnFailure<ServerErrorsFailureClass> for CustomOnFailure {
    fn on_failure(
        &mut self,
        failure_classification: ServerErrorsFailureClass,
        latency: Duration,
        _span: &Span,
    ) {
        let latency_ms = latency.as_secs_f64() * 1000.0;
        error!(
            target: "serve::http",
            classification = %failure_classification,
            latency_ms = format_args!("{:.2}ms", latency_ms),
            "🔥 HTTP request processing failed prematurely"
        );
    }
}

/// Helper function to create the customized TraceLayer for Axum router
pub fn create_http_trace_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    CustomMakeSpan,
    CustomOnRequest,
    CustomOnResponse,
    DefaultOnBodyChunk,
    DefaultOnEos,
    CustomOnFailure,
> {
    TraceLayer::new_for_http()
        .make_span_with(CustomMakeSpan)
        .on_request(CustomOnRequest)
        .on_response(CustomOnResponse)
        .on_failure(CustomOnFailure)
}

fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    if let Some(x_forwarded_for) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if let Some(client_ip) = x_forwarded_for.split(',').next() {
            let trimmed = client_ip.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    if let Some(x_real_ip) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        return Some(x_real_ip.trim().to_string());
    }
    None
}
