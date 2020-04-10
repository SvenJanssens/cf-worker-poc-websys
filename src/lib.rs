extern crate cfg_if;
extern crate wasm_bindgen;

#[macro_use]
mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

/// A struct to hold some data from the github Branch API.
///
/// Note how we don't have to define every member -- serde will ignore extra
/// data when deserializing
#[derive(Debug, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub commit: Commit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub commit: CommitDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitDetails {
    pub author: Signature,
    pub committer: Signature,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Signature {
    pub name: String,
    pub email: String,
}

#[wasm_bindgen]
pub async fn run(repo: String) -> Result<JsValue, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");

    let url = format!("https://api.github.com/repos/{}/branches/master", repo);

    let request = Request::new_with_str_and_init(&url, &opts)?;

    request
        .headers()
        .set("Accept", "application/vnd.github.v3+json")?;
    
    request
        .headers()
        .append("User-Agent", "cf-worker-poc-websys")?;

    // Await the fetch() promise
    // Credits for the solution with GLOBAL_WEB_CONTEXT go to GlobiDev @ https://github.com/Globidev/reqwest/blob/wasm-webworkers/src/wasm/client.rs
    // As the standard solution on https://rustwasm.github.io/docs/wasm-bindgen/examples/fetch.html doesn't work for CloudFlare workers,
    // because the web_sys::Window methods aren't supported there (as no window is involved for the Cloudflare Workers)
    let p = GLOBAL_WEB_CONTEXT.with(|g| {
        match g {
            WebContext::Window(w) => w.fetch_with_request(&request),
            WebContext::Worker(w) => w.fetch_with_request(&request),
        }
    });

    let resp_value = JsFuture::from(p).await?;
 
    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let json = JsFuture::from(resp.json()?).await?;

    // Use serde to parse the JSON into a struct.
    let branch_info: Branch = json.into_serde().unwrap();

    // Send the `Branch` struct back to JS as an `Object`.
    Ok(JsValue::from_serde(&branch_info).unwrap())
}

thread_local! {
    static GLOBAL_WEB_CONTEXT: WebContext = WebContext::new();
}

#[derive(Debug)]
enum WebContext {
    Window(web_sys::Window),
    Worker(web_sys::WorkerGlobalScope),
}

impl WebContext {
    fn new() -> Self {
        #[wasm_bindgen]
        extern "C" {
            type Global;

            #[wasm_bindgen(method, getter, js_name = Window)]
            fn window(this: &Global) -> JsValue;

            #[wasm_bindgen(method, getter, js_name = WorkerGlobalScope)]
            fn worker(this: &Global) -> JsValue;
        }

        let global: Global = js_sys::global().unchecked_into();

        if !global.window().is_undefined() {
            Self::Window(global.unchecked_into())
        } else if !global.worker().is_undefined() {
            Self::Worker(global.unchecked_into())
        } else {
            panic!("Only supported in a browser or web worker");
        }
    }
}