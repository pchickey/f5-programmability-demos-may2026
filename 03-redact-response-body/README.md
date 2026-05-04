# 03: Redact Response Body

As always, the program lives in
[`src/main.rs`](https://github.com/pchickey/f5-programmability-demos-may2026/blob/main/03-redact-response-body/src/main.rs).

This program is a bit more interesting than the last two!

Instead of just calculating a response, this program:

1. makes a HTTP request to another system
2. reads the complete response body
3. decodes the response body from JSON to a structure,
4. redacts one of the fields in the decoded structure,
5. encodes back to JSON as a new body,
4. and then returns that response.

You can think of this as a web proxy that modifies intercepts and modifies the
response.

The [`wstd::http::Body`] type has methods for creating and reading bodies.

[`wstd::http::Body`]: https://docs.rs/wstd/latest/wstd/http/struct.Body.htm

The [`serde`] crate's `Serialize` and `Deserialize` traits are the key to
decoding and encoding as JSON in this example, and they are very powerful in
general. The `serde` crate is a framework for generic serialization and
deserialization, and a wide range of crates like [`serde_json`], `serde_yaml`,
`serde_cbor` etc.

[`serde_json`]

`Body` provides the methods [`Body::json`] which collects an entire body
(do just this part with [`Body::contents`]) and then uses `serde_json` to
decode the body to any struct or enum that impls `Deserializable`.

[`Body::from_json`] does the opposite operation, it takes any struct or
enum which impls `Serializable`, uses `serde_json` to encode it to JSON,
and then constructs a body frmo those contents.

[`serde`]: https://docs.rs/serde/latest/serde/A
[`Body::json`]: https://docs.rs/wstd/latest/wstd/http/struct.Body.html#method.json
[`Body::contents`]: https://docs.rs/wstd/latest/wstd/http/struct.Body.html#method.contents
