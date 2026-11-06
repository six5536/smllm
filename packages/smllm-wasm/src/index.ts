// smllm-wasm for browsers and Node (the web build): call `init()` (or
// `initSync`) once, then use `Engine`. The string API is `smllm-wasm/raw`.
import load, { Engine as Raw, initSync } from "../wasm/web/smllm_wasm.js";
import { Engine as Base } from "./wrap.js";

type Input = Parameters<typeof load>[0];

// sokf-implements: WASM-6_AC-2
/**
 * Load the engine's WebAssembly once. Without an argument, a browser fetches
 * the package's file, and Node.js, which can't fetch a `file:` URL, reads it.
 */
export default async function init(input?: Input): ReturnType<typeof load> {
  const node = (globalThis as { process?: { versions?: { node?: string } } }).process?.versions?.node;
  if (input === undefined && node) {
    // A variable keeps a bundler for the browser from resolving `node:fs`.
    const fs = "node:fs/promises";
    const { readFile } = (await import(/* @vite-ignore */ fs)) as typeof import("node:fs/promises");
    const file = new URL("../wasm/web/smllm_wasm_bg.wasm", import.meta.url);
    return load({ module_or_path: await readFile(file) });
  }
  return load(input);
}

export { initSync };
export type * from "./types.js";
export { memoryStorage } from "./storage.js";
/** smllm's engine over compiled machines (`smllm compile` output). */
export class Engine extends Base {
  protected static override Raw = Raw;
}
