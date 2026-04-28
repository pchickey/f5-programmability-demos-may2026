use anyhow::anyhow;
use wstd::http::{Body, Client, Error, Request, Response, StatusCode};
use wstd::task::sleep;
use wstd::time::Duration;

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
        // Making a Body out of another Body is much more useful when you're
        // using it to forward a response body:
        "/forward_response_body" => {
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
            // all of the upstream headers, but lets just forward
            // "x-example-origin" as "x-upstream-example"
            let mut resp = Response::builder();
            if let Some(version) = upstream_parts.headers.get("x-example-origin") {
                resp = resp.header("x-upstream-example", version);
            }
            // And we just put `upstream_body` here, and wstd will forward it
            // efficiently.
            Ok(resp.body(upstream_body)?)
        }
        // HTTP bodies are streams, which means they can be a sequence of
        // chunks. Wstd can produce and consume bodies chunk by chunk as well.
        // This example shows producing a body in chunks.
        //
        // Streaming behavior is fully supported in BIG-IP, but not yet in
        // NGINX. This example will produce 4 separate data frames sent 0.5s
        // apart in BIG-IP, but it will produce a single data frame sent after
        // 2s in NGINX.
        "/stream_response_body" => {
            use futures_lite::{stream, StreamExt};
            // Start with something to iterate through
            let dogs = vec!["Gussie", "Willa", "Sparky", "Benny"];
            // stream::iter turns the Vec into a Stream. StreamExt::then
            // allows us to apply an async closure to each item in the Stream.
            let stream = stream::iter(dogs).then(|dog| async move {
                sleep(Duration::from_millis(500)).await;
                format!("Hello, {dog}\n")
            });
            // Finally, we can make a body out of the Stream.
            Ok(Response::new(Body::from_stream(stream)))
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(format!("not found: {path:?}").into())?),
    }
}
