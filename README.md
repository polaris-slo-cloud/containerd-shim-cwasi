# CWASI Containerd shim

WIP: Should be able to spawn containers and wasm modules

Based: https://github.com/keniack/runwasi/tree/main/crates/containerd-shim-wasmedge


CWASI containerd shim is a lightweight and portable way to run containers using WebAssembly modules. It provides a secure and sandboxed environment for running untrusted code, making it ideal for use cases such as running untrusted code in a serverless environment.

## Features

* Wasm dependencies automatic loading
* Local function communication optmization via unix socket / pipes
* Remote function communication via message broker

## Prerequisites

* Rust 
* Containerd
* Wasmedge

## Installation
```
cargo build --release
```

Copy binary to $PATH
```
sudo cp target/release/containerd-shim-cwasi-v1 /usr/local/bin/containerd-shim-cwasi-v1
```

## Usage
```
Example 1
sudo crictl pull docker.io/wasmedge/example-wasi:latest
sudo ctr -n k8s.io run  --runtime=io.containerd.cwasi.v1 --annotation cwasi.secondary.function=true --net-host=true docker.io/wasmedge/example-wasi:latest cwasi /wasi_example_main.wasm 50000000

Example 2

sudo crictl pull docker.io/keniack/alice-wasm-app:latest
sudo crictl pull docker.io/keniack/my_math_lib:latest
sudo ctr -n k8s.io run --rm --runtime=io.containerd.cwasi.v1 --annotation cwasi.secondary.function=true --net-host=true docker.io/keniack/alice-wasm-app:latest cwasi /alice-wasm-app.wasm 5 10

```

## Contributing

Contributions are welcome! Please fork this repository and open a pull request with your changes.

## License

The Cwasi Shim is licensed under the Apache License, Version 2.0. See link for the full license text.