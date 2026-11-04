# Contributing to smllm

This guide explains how to report a bug or ask for a feature, and how to build smllm from this repository. This repository is a read-only mirror of smllm. An export from the project's own repository creates one commit here for each release, and nobody commits to the mirror directly.

- [Report a bug or ask for a feature](#report-a-bug-or-ask-for-a-feature)
- [Build smllm](#build-smllm)

## Report a bug or ask for a feature

Open an issue with the bug report template or the feature request template. A pull request can't be merged here, because the next release replaces the whole tree. When you have a change in mind, describe it in an issue, and include a patch there if you have one.

## Build smllm

The project pins its toolchains in `rust-toolchain.toml` and `.mise.toml`, which [mise](https://mise.jdx.dev/) installs. These commands install the toolchains and build the CLI:

```sh
mise install
cargo build --locked -p smllm
```

The build takes `agent-harness-adapter-core`, the crate that installs smllm into each agent's harness, from crates.io. Its source is in the [agent-harness-adapter](https://github.com/six5536/agent-harness-adapter) repository.

To build the npm packages as the release does, install the JavaScript workspace, build the Linux and macOS binaries, and build the WebAssembly package:

```sh
npm install
npm run build:all
npm run build:wasm
```

`npm run build:all` builds the four Linux and macOS binaries. The release builds the Windows binary on Windows. `npm run build:wasm` installs the nightly toolchain that `scripts/build-wasm.mjs` pins. It also needs `wasm-bindgen` at the version of the `wasm-bindgen` crate in `Cargo.lock`.

The release workflow is `.github/workflows/publish.yml`. It builds the five binaries and the WebAssembly package, smoke-tests the binaries, publishes the crates and the packages, and creates the GitHub release with the notes from the tag's message. The GitHub releases hold the notes of every release, and they are the project's changelog.
