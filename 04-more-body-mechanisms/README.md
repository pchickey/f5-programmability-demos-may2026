# 04: More Body Mechanisms

This example uses a really basic router based on a Rust match of the
path segment of the request
[Uri](https://docs.rs/http/latest/http/uri/struct.Uri.html).

Then, based on the path, it shows a couple different useful methods for
working with the [`Body`] type:

1. Gathering the body as a string (which, in Rust, implies UTF-8 encoding),
   and then constructing the body from a string.
   Try this one out with:
   ```sh
   $ curl 10.1.1.4:8000 -d "Hello, F5 Wasm Programmability!"
   Hello, World!
   Request body was:
   Hello, F5 Wasm Programmability!
   ```

2. Constructing a body from raw bytes - in this case, it constructs the
   smallest possible WebAssembly module, in binary encoding. Try this one out
   with:
   ```sh
   $ curl 10.1.1.4:8000/vec_of_u8 | wasm-tools print
   (module)
   ```
   wasm-tools print is a command for taking the Wasm binary format and
   printing it as the Wasm text format (which is called wat)

3. Constructing a response body from a request body. The wasi-http interface
   goes to lengths to make it possible to forward the body of a http request
   or response as the body of another request or response as efficiently as
   possible, in particular avoiding an unneeded copy into and out of the
   WebAssembly sandbox.
   ```sh
   $ curl 10.1.1.4/request_body -d "Hello, echo server"
   Hello, echo server
   ```
4. Constructing a response body from another response body. This is a much
   more useful case than the echo server in the previous case, because a lot
   of the time a proxy just wants to forward a body unmodified.
   This example makes a request to the example origin (introduced in the 03
   example) and forwards exactly what it gets as the response. **Note that
   because it contacts the example origin, BIG-IP cannot run this
   example**^1

5. Show that bodies are streaming - in principle. You can recieve or send a
   body chunk by chunk, waiting any amount of time in between. In BIG-IP,
   streaming bodies are fully implemented, so if you run this example in
   BIG-IP you will see each line of the output arrive 0.5 seconds apart:
   ```sh
   $ curl 10.1.1.4:3000/stream_response_body
   Hello, Gussie
   Hello, Willa
   Hello, Sparky
   Hello, Benny
   ```
   Yes, the author has 4 dogs, and yes, that is too many. But we love them.

   If you run the exact same code in NGINX, the Wasm will still "send" the
   body chunks 0.5 seconds apart, but NGINX will buffer util it has the
   complete body and send it all at once. So, when you use curl, it will wait
   2 seconds and then the entire output will arrive. This is a bug specific to
   the NGINX Wasm integration, and a fix is in progress!
   ```sh
   $ curl 10.1.1.4:8000/stream_response_body
   Hello, Gussie
   Hello, Willa
   Hello, Sparky
   Hello, Benny
   ```

6. If you ask for a path that isnt in the router, the response will return
   status 404 (`StatusCode::NOT_FOUND`) and an informative body.
   Try asking curl to print the status code after the body:

   ```sh
   $ curl -w "\n%{http_code}" 10.1.1.4:8000/something
   not found: "/something"
   404
   ```

[`Body`]: https://docs.rs/wstd/latest/wstd/http/struct.Body.htm


[^1]: BIG-IP is running on a different subnet than NGINX in the demo setup.
    For extra credit, you could compile the example origin and run it on
BIG-IP on another port. In the example origin folder (`99
Example Origin`) run the Build task (or terminal `../common/build.sh`) and then tell
BIG-IP to run it on an alternative port 3001 using a terminal in `99 Example
Origin`:

```sh
$ curl http://10.1.1.4:9001/services?name=example-origin&port=3001
```
and then you can change the upstream request address to
`10.254.1.2:3001/people.json`, and then run the `Run on BIG-IP` task in `04
More Body Mechanisms`
