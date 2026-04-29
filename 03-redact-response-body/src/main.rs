use serde::{Deserialize, Serialize};
use wstd::http::{Body, Client, Error, Request, Response};

#[wstd::http_server]
async fn main(_req: Request<Body>) -> Result<Response<Body>, Error> {
    // This service is just going to be modifying the response body from
    // another service. Here, we're making a request to the Example Origin
    // service's /people.json endpoint.
    //
    // The /people.json endpoint is a toy example which gives out too much
    // Personally Identifiable Information (PII) in the form of social
    // security numbers (SSNs). (The ones actually present in this demo are,
    // indeed, fake.)
    //
    // This demo is writing a filter which redacts those SSNs in the response.
    let mut people_resp = Client::new()
        .send(Request::get("http://10.1.1.4:8001/people.json").body(())?)
        .await?;

    // /people.json returns a JSON structure where there is a list of dicts,
    // and each dict has string fields `firstname`, `lastname`, `city`, `ssn`.
    // Using Rust's serde ecosystem, we can write a `struct Person` that has
    // those fields, and the `#[derive(Serialize, Deserialize)]` means serde
    // will know how to decode that struct from JSON and encode it to JSON.
    #[derive(Serialize, Deserialize)]
    struct Person {
        firstname: String,
        lastname: String,
        city: String,
        ssn: String,
    }

    // We can collect the body in the response and then decode it from JSON
    // into a list of persons:
    let mut people: Vec<Person> = people_resp.body_mut().json().await?;
    // Then, for each person, we redact the ssn field:
    for person in people.iter_mut() {
        person.ssn = "REDACTED".to_owned();
    }
    // Then we replace the existing body with a new one, formed by encoding
    // the now-redacted list of persons back to JSON:
    *people_resp.body_mut() = Body::from_json(&people)?;

    Ok(people_resp)
}
