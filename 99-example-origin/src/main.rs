use wstd::http::{Body, HeaderName, Request, Response, Result, StatusCode};
#[wstd::http_server]
async fn main(req: Request<Body>) -> Result<Response<Body>> {
    match req.uri().path() {
        "/show_headers" => show_headers(&req),
        "/image.png" => image(&req),
        "/people.json" => people(&req),
        "/metrics.json" => metrics(&req),
        "/reader" => reader(req).await,
        p => Ok(Response::builder()
            .status(404)
            .body(format!("not found: {p:?}").into())
            .unwrap()),
    }
}

fn show_headers(req: &Request<Body>) -> Result<Response<Body>> {
    let mut resp_body = Vec::new();
    let mut resp = Response::new(().into());
    for (req_header, value) in req.headers() {
        if let Some(suffix) = req_header
            .as_str()
            .to_lowercase()
            .strip_prefix("x-response-header-")
        {
            let name = HeaderName::from_lowercase(suffix.as_bytes()).unwrap();
            resp_body.push(format!("response header {name}: {value:?}"));
            resp.headers_mut().insert(name, value.clone());
        } else {
            resp_body.push(format!("request header {req_header}: {value:?}"));
        }
    }

    *resp.body_mut() = resp_body.join("\n").into();
    Ok(resp)
}

const IMAGE: &[u8] = include_bytes!("mthood.png");

fn image(_req: &Request<Body>) -> Result<Response<Body>> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "image/png")
        .body(IMAGE.into())
        .unwrap())
}

fn people(_req: &Request<Body>) -> Result<Response<Body>> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .header("x-example-origin", env!("CARGO_PKG_VERSION"))
        .body(serde_json::to_string(&serde_json::json!(
        [
            { "firstname": "Pat", "lastname": "Hickey", "city": "Portland", "ssn": "234-56-7890" },
            { "firstname": "Chris", "lastname": "Fallin", "city": "Sunnyvale", "ssn": "123-45-6789" },
            { "firstname": "Nick", "lastname": "Fitzgerald", "city": "Portland", "ssn": "987-65-4321" },
        ]
        )).unwrap().into())
        .unwrap())
}

const METRICS: &str = include_str!("metrics.json");

fn metrics(_req: &Request<Body>) -> Result<Response<Body>> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(METRICS.into())
        .unwrap())
}

async fn reader(mut req: Request<Body>) -> Result<Response<Body>> {
    let req_body = req.body_mut().str_contents().await?;
    let mut body =
        format!("this response is from the origin server.\nrequest body was: {req_body:?}\n");
    if req_body.contains("etc/passwd") {
        body += "the root passwd is hunter2\n";
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/plain")
        .body(body.into())
        .unwrap())
}
