use anyhow::anyhow;
use wstd::http::{Body, Client, Error, Request, Response, StatusCode};

#[wstd::http_server]
async fn main(req: Request<Body>) -> Result<Response<Body>, Error> {
    let path = req
        .uri()
        .path_and_query()
        .ok_or_else(|| anyhow!("missing path_and_query, which should always be populated"))?
        .path();
    match path {
        // You've already seen using a str as a body:
        "/" => Ok(Response::new("Hello, world!\n".into())),
        // We can also make a Body out of bytes, such as a Vec<u8>, &[u8], or
        // the bytes::Bytes types
        "/vec_of_u8" => {
            let some_bytes: Vec<u8> = vec![0x00u8, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
            Ok(Response::builder()
                .header("content-type", "application/octet-stream")
                .body(Body::from(some_bytes))?)
        }
        // We can make a Body out of another Body. When used for either
        // echoing the request body (not very useful) or forwarding another
        // response body (very useful for proxies), wstd will optimize this
        // operation to avoid copying the body contents into and out of the
        // wasm sandbox.
        "/request_body" => {
            let (_req_parts, req_body) = req.into_parts();
            Ok(Response::new(req_body))
        }
        "/forward_response_body" => {
            // Here's a really powerful tool this is your first time seeing:
            // the http Client!
            // Lets make a request to somewhere - in this case, we have an
            // "example origin" application running already on our NGINX.
            let upstream_resp = Client::new()
                .send(Request::get("http://10.1.1.4:8001/people.json").body(())?)
                // HTTP requests are an async operation, so we await the
                // completion here:
                .await?;
            let (upstream_parts, mut upstream_body) = upstream_resp.into_parts();

            if upstream_parts.status != StatusCode::OK {
                // Handle the error case. Here, we want to gather the upstream
                // body text and process it as a &str, using .str_contents(). Bodies are a stream,
                // so completing str_contents() is an async opoeration that
                // must await completion.
                let upstream_body_contents = upstream_body.str_contents().await;
                // Put the relevant information into our error message:
                let error_body = format!(
                    "upstream failed with status {}: {upstream_body_contents:?}",
                    upstream_parts.status,
                );
                // Return an internal server error with informative body:
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(error_body.into())?);
            }

            // Happy case: we can make a new response. We could forward any or
            // all of the upstream headers, but lets just forward Server if
            // its available.
            let mut resp = Response::builder();
            if let Some(upstream_server) = upstream_parts.headers.get("server") {
                resp = resp.header("upstream-server", upstream_server);
            }
            // And we just put `upstream_body` here, and wstd will forward it
            // efficiently.
            Ok(resp.body(upstream_body)?)
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(format!("not found: {path:?}").into())?),
    }
}
