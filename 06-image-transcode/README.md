# 06: Image Transcode

This example is a quick demonstration of how the open-source Rust ecosystem
makes it very easy to do things that are very difficult in existing F5
Programmability systems such as iRules or njs.

The example origin is hosting a PNG encoded image at
`10.1.1.4:8001/image.png`. This program fetches from the example origin,
decodes the body using the [`image`] crate's [`ImageReader`], and encodes it
as a [WebP], which typically has 20-25% better compression than PNG.

You'll need to use the UDF buttons to open the `NGINX Default Service` in your
browser to see this one. In Chrome, you can open up the web inspector
(right click, Inspect) and look in the Network tab to see that Chrome got a
smaller webp than it gets from `/image.png` in the UDF `NGINX Example Origin`.

[`image`]: https://docs.rs/image/latest/image/index.html
[`ImageReader`]: https://docs.rs/image/latest/image/struct.ImageReader.html
[WebP]: https://en.wikipedia.org/wiki/WebP


**Note that because this uses the example origin, it will not work properly on
BIG-IP** unless you follow the notes at the very bottom of `04 More Body
Mechanisms`'s README.md to put the example origin onto BIG-IP as well, and
tweak the source text to fetch from there instead.


