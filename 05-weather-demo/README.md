# Weather Demo

This demo demonstrates some important concepts:
1. You can make an arbitrary number of HTTP Client requests as part of
   fulfilling a single HTTP Server request
2. You can use open source crates from Rust's ecosystem to take care of JSON
   and query string encoding
3. Concurrency using async Rust programming

All of these concepts are best elucidated by the comments in the program's
source code.

## What it does

The actual demo is to provide a weather report for a given city, where the
city name might be ambigious.

For example, a request such as:

```
curl http://10.1.1.14:8000?city=portland
```

will first fetch from a geocoding API to find locations named "portland", and
sort them by population.

You can limit how many locations by adding a query paramter "count". When not
provided, it defaults to 10.

```
curl http://10.1.1.14:8000?city=portland&count=2
```

It will then fetch from a weather API according to each lat/lon coordinate in
that list of cities. These fetches happen concurrently.

Finally, the server responds with the complete list of locations and their
current weather, encoded in JSON.

## Concurrency and Async Rust

Async Rust ought to be pretty familiar if you ever used async/await in the C#
language, but it usually is one of the more difficult topics in Rust to learn.

If you are just learning the Rust, make sure you have a solid understanding of
ownership and traits, then read two or three "beginer guides" to Rust you find
by googling, and finally, block out a couple of hours to carefully read (and
maybe even program along at home!) this incredible article from Amos:
[https://fasterthanli.me/articles/request-coalescing-in-async-rust](https://fasterthanli.me/articles/request-coalescing-in-async-rust)

