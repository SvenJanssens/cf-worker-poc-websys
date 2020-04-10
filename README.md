# A proof-of-concept for using web-sys-fetch in a Cloudflare Worker

This proof-of-concept takes the [web-sys/fetch example](https://rustwasm.github.io/docs/wasm-bindgen/examples/fetch.html) at the wasm-bindgen website and adapts it so it can be used as a Cloudflare worker.

To generate the required files for a Cloudflare worker, the excellent template for kick starting a Cloudflare worker project using
[`wasm-pack`](https://github.com/rustwasm/wasm-pack) was used.

## Findings

- The example at the wasm-bindgen website uses the websys::window() function to send out the request to the API. However, websys::Window context is not usable in a Cloudflare Worker (as there is no window). In this case the websys::Worker context should be used.
- The GitHub API now requires for the calling logic to send a HTTP Header "UserAgent".
