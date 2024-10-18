# Arguments of non-standard wasm types in wasm function

- Build `wasm-lib`

  ```bash
  cargo build -p wasm-lib --target wasm32-wasi --release
  ```

  If the command runs successfully, `wasm-lib.wasm` can be found in the directory of `./target/wasm32-wasi/release/`.

- Build and run `call-wasm-lib`

  ```bash
  cargo run -p call-wasm-lib -- ./target/wasm32-wasi/release/wasm_lib.wasm 5
  ```

  If the command runs successfully, then the following message is printed out on the screen:

  ```bash
    Run bindgen -- say: hello bindgen funcs test 5
    Run bindgen -- say FAILED abc
    Run bindgen -- create_line: {"points":[{"x":2.5,"y":7.8},{"x":2.5,"y":5.8}],"valid":true,"length":2.0,"desc":"A thin red line"}
    Run bindgen -- obfusticate: N dhvpx oebja sbk whzcf bire gur ynml qbt
    Run bindgen -- sha3_digest: [87, 27, 231, 209, 189, 105, 251, 49, 159, 10, 211, 250, 15, 159, 154, 181, 43, 218, 26, 141, 56, 199, 25, 45, 60, 10, 20, 163, 54, 211, 195, 203]
    Run bindgen -- keccak_digest: [126, 194, 241, 200, 151, 116, 227, 33, 216, 99, 159, 22, 107, 3, 177, 169, 216, 191, 114, 156, 174, 193, 32, 159, 246, 228, 245, 133, 52, 75, 55, 27]
  ```
