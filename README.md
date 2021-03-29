# WebTonic

Browser enables websocket tunneling of gRPC messages.

## Testing

This repository implements a set of small test crates.
Running these tests requieres to [install](https://rustwasm.github.io/wasm-pack/installer/)
`wasm-pack`.

To run the tests, first start the server:

```bash
RUST_LOG=info cargo run -p server-test
```

Then, after the server is built and running, the client tests can be run.

To test in firefox, run:

```bash
wasm-pack test --firefox --headless client-test
```

To test in chrome, run:

```bash
wasm-pack test --chrome --headless client-test
```