use image::{ImageFormat, ImageReader};
use std::io::Cursor;
use wstd::http::{Body, Client, Error, Request, Response};

#[wstd::http_server]
async fn main(req: Request<Body>) -> Result<Response<Body>, Error> {
    // See if the string "upside_down" is present in thw query:
    let upside_down = req
        .uri()
        .query()
        .unwrap_or_default()
        .contains("upside_down");

    // Fetch a png from the origin
    let mut origin_resp = Client::new()
        .send(Request::get("http://10.1.1.4:8001/mthood.png").body(())?)
        .await?;

    // Collect the body as a slice of bytes:
    let contents = origin_resp.body_mut().contents().await?;

    // Transcode to a webp:
    let webp_contents = transcode(contents, upside_down)?;

    // Return a response with content-type for webp:
    let response = Response::builder()
        .header("content-type", "image/webp")
        .body(Body::from(webp_contents))?;

    Ok(response)
}

fn transcode(contents: &[u8], upside_down: bool) -> Result<Vec<u8>, Error> {
    // image crate can read the image bytes in, and guess the format:
    let mut image = ImageReader::new(Cursor::new(contents))
        .with_guessed_format()?
        .decode()?;

    // Flip if desired:
    if upside_down {
        image = image.flipv();
    }

    // Write the output as a WebP
    let mut out = Cursor::new(Vec::new());
    image.write_to(&mut out, ImageFormat::WebP)?;

    // Unwrap the Cursor<Vec<u8>> to Vec<u8>
    Ok(out.into_inner())
}
