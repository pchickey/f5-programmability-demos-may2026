# 01: Hello World

This is the simplest possible program that can run in our programmability
solution: an HTTP server that, no matter what request it recieves, always
returns a 200 OK response with tbe body "Hello World!\n".

## The program's source code

The entire program lives in
[`src/main.rs`](https://github.com/pchickey/f5-programmability-demos-may2026/blob/main/01-hello-world/src/main.rs).

## Build and Run this program

There is a VS Code Task to build this program. In the top `Terminal` menu, select `Run Task`

![VSCode Terminal Menu Run Task](../.images/run-task.png)

Select `Run in NGINX 01: Hello World`. This will perform a build, then load the built
WebAssembly into NGINX.

Then, from the `Terminal` menu, open a terminal. Send a request to the
service, which is running on `10.1.1.4:8000`:

```sh
$ curl 10.1.1.4:8000
Hello, World!
```
