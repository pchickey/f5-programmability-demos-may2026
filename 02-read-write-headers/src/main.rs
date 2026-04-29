use std::str::FromStr;
use wstd::http::{Body, Error, HeaderName, HeaderValue, Request, Response};

#[wstd::http_server]
async fn main(req: Request<Body>) -> Result<Response<Body>, Error> {
    // Create a response body by building up a String from parts:
    let mut resp_body_contents: String = "<!DOCTYPE html>\n<html>\n<body>\n".to_owned();
    resp_body_contents.push_str("<h1>Request Contents:</h1>\n");
    resp_body_contents.push_str("<h2>Headers:</h2>\n");

    // Iterate through the request's headers. Put those into the response
    // body:
    for (header_name, header_value) in req.headers() {
        // The header_name value gets interpolated as Display with plain {}
        // the header_value value gets interpolated as Debug with {:?} because it may
        // contain weird characters.
        resp_body_contents.push_str(&format!("<p>{header_name}: {header_value:?}</p>\n"));
    }

    // Conditional logic on the user agent.
    if let Some(user_agent) = req.headers().get("User-Agent") {
        let user_agent = user_agent.to_str()?;
        let user_agent = user_agent.to_lowercase();
        if user_agent.contains("curl") {
            resp_body_contents.push_str("<h3>The user agent was curl</h3>\n");
        }
    }

    resp_body_contents.push_str("</body>\n</html>\n");
    let resp_body: Body = resp_body_contents.into();
    let mut resp = Response::new(resp_body);

    // We can modify the headers of the Response as well:
    resp.headers_mut().insert(
        // These functions are fallible: if you used an illegal character
        // (like \n) in the name or value, they will fail. The ? turns their
        // failure into a failure of the entire function ? is used in.
        HeaderName::from_str("Content-Type")?,
        HeaderValue::from_str("text/html; charset=utf-8")?,
    );

    Ok(resp)

    // Things to try :
    // * If you change the case in the conditional to `req.headers().get("uSER-aGENT")` it still works,
    //   because the HeaderMap structure is case insensitive.
    //
    // * There are constants for common (RFC) header names. Run `cargo add
    //   http` to depend on Rust's http crate, then try replacing
    //   `HeaderName::from_str("Content-Type")?` with
    //   `http::header::CONTENT_TYPE`.
}
