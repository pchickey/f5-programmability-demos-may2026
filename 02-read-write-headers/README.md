# 02: Read and Write Headers

Once again, the entire program lives in
[`src/main.rs`](https://github.com/pchickey/f5-programmability-demos-may2026/blob/main/02-read-write-headers/src/main.rs).

This program demonstrates reading some of the headers from the HTTP request,
and writing some headers to the HTTP response.

It also shows how to construct a response body by appending strings, and some
basics of Rust control flow: `for` and `if`.

## Build and Run

Once again, use `Terminal` -> `Run Task`, and this time select `Run in NGINX
02 Read Write Headers`.

Then you can use `curl 10.1.1.4:8000` to make a request.

Since the response is a web page, you can also try using the UDF buttons to
access `NGINX DEFAULT SERVICE`.
