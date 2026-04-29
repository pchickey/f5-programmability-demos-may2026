use wstd::http::{Body, Error, Request, Response};

#[wstd::http_server]
async fn main(_req: Request<Body>) -> Result<Response<Body>, Error> {
    // Contents of the body are just a str here.
    let resp_body_contents: &str = "Hello, world!\n";
    // Convert those contents into a Body.
    let resp_body: Body = resp_body_contents.into();
    // Create a Response that contains that Body.
    let resp = Response::new(resp_body);
    // Return it in Ok, because this function was successful. If you returned
    // an error, the HTTP Server would hang up.
    Ok(resp)

    // Ideas for further exploration:
    // * Many different things can convert to a Body:
    //   https://docs.rs/wstd/latest/wstd/http/struct.Body.html#trait-implementations
    // * You could instead return an error, replacing the Ok with
    //   `Err(Error::msg("something terrible occured"))`.
    //   When Wasm returns an Error to NGINX instead of a Response, NGINX
    //   itself will respond with an empty 500, and log the contents of the
    //   Error. You'll need to look in the nginx error logs to debug the
    //   Error.
}
