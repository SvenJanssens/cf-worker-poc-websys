addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

/**
 * Fetch and log a request
 * @param {Request} request
 */
async function handleRequest(request) {
  const { run } = wasm_bindgen;
  await wasm_bindgen(wasm)

  const data = await run("rustwasm/wasm-bindgen")

  console.log(data);
  console.log("The latest commit to the wasm-bindgen %s branch is:", data.name);
  console.log("%s, authored by %s <%s>", data.commit.sha, data.commit.commit.author.name, data.commit.commit.author.email);

  return new Response("", {status: 200})
}