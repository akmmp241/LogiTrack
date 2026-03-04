use crate::app::AppState;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use http_body_util::BodyExt;
use reqwest::Method;
use uuid::Uuid;

pub async fn reverse_proxy(state: AppState, req: Request<Body>, target: String) -> Response {
    // for tracing the request in the future
    let correlation_id = Uuid::new_v4().to_string();

    let path = req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("");
    let url = format!("{}{}", target, path);

    let method = Method::from_bytes(req.method().as_str().as_bytes()).unwrap_or(Method::GET);

    let mut headers = Vec::new();
    for (key, value) in req.headers().iter() {
        if let (Ok(name), Ok(val)) = (
            reqwest::header::HeaderName::from_bytes(key.as_str().as_bytes()),
            reqwest::header::HeaderValue::from_bytes(value.as_bytes()),
        ) {
            headers.push((name, val));
        }
    }

    let body_bytes = match req.into_body().collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Failed to read request body"))
                .unwrap();
        }
    };

    let mut request_builder = state.client.request(method, url);

    for (key, value) in headers {
        request_builder = request_builder.header(key, value);
    }

    request_builder = request_builder
        .header("x-correlation-id", &correlation_id)
        .body(body_bytes);

    let response = match request_builder.send().await {
        Ok(res) => res,
        Err(err) => {
            tracing::error!("Upstream error: {:?}", err);
            return Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from("Upstream error"))
                .unwrap();
        }
    };

    let status = response.status();
    let resp_headers = response.headers().clone();
    let stream = response.bytes_stream();

    let mut axum_response = Response::builder().status(status);

    for (key, value) in resp_headers.iter() {
        axum_response = axum_response.header(key, value);
    }

    axum_response.body(Body::from_stream(stream)).unwrap()
}
