// Only the ABI lives here. Every conversation decision lives in Rust.
export class Engine {
  constructor(exports) { this.wasm = exports; }
  static async load(url = new URL("karabasan.wasm", import.meta.url)) {
    const response = await fetch(url);
    if (!response.ok) throw new Error(`WASM yüklenemedi (${response.status}). Önce make web çalıştır.`);
    const { instance } = await WebAssembly.instantiate(await response.arrayBuffer(), {});
    return new Engine(instance.exports);
  }
  response() {
    const w = this.wasm;
    const bytes = new Uint8Array(w.memory.buffer, w.chat_response_ptr(), w.chat_response_len());
    return JSON.parse(new TextDecoder().decode(bytes));
  }
  start(seed) { this.wasm.chat_new(seed >>> 0); return this.response(); }
  submit(input) {
    const bytes = new TextEncoder().encode(input);
    const ptr = this.wasm.chat_alloc(bytes.length);
    try {
      new Uint8Array(this.wasm.memory.buffer, ptr, bytes.length).set(bytes);
      this.wasm.chat_submit(ptr, bytes.length);
      return this.response();
    } finally { this.wasm.chat_free(ptr, bytes.length); }
  }
}
