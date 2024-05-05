# CWASI Containerd shim

![Build](https://github.com/polaris-slo-cloud/containerd-shim-cwasi/actions/workflows/rust.yml/badge.svg)
![Repo Updated Badge](https://badges.strrl.dev/updated/polaris-slo-cloud/containerd-shim-cwasi)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://github.com/polaris-slo-cloud/containerd-shim-cwasi/blob/main/LICENSE)
[![experimental](http://badges.github.io/stability-badges/dist/experimental.svg)](http://github.com/badges/stability-badges)

CWASI is a WebAssembly OCI-compliant runtime shim that identifies and selects the best inter-function data exchange approach based on serverless functions location. We got inspired by [RunWasi Wasmedge](https://github.com/containerd/runwasi).

<p align="center">
  <img src="images/cwasi_architecture.svg" width="35%" height="35%">
</p>

## Features

* Novel model for serverless function communication
* Wasm modules static linking via function embedding
* Co-hosted function communication optimization via kernel buffer
* Remote function communication via message broker

## Prerequisites

* Rust 
* Containerd 
* Wasmedge -v 0.13.3
* Redis
* Cri-tools for execution

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
export REDIS_IP=<redis-ip>
sudo crictl pull docker.io/wasmedge/example-wasi:latest
sudo ctr -n k8s.io run  --runtime=io.containerd.cwasi.v1 --annotation cwasi.secondary.function=true --net-host=true docker.io/wasmedge/example-wasi:latest cwasi /wasi_example_main.wasm 50000000

Example 2
export REDIS_IP=<redis-ip>
sudo crictl pull docker.io/keniack/alice-wasm-app:latest
sudo crictl pull docker.io/keniack/my_math_lib:latest
sudo ctr -n k8s.io run --rm --runtime=io.containerd.cwasi.v1 --annotation cwasi.secondary.function=true --net-host=true docker.io/keniack/alice-wasm-app:latest cwasi /alice-wasm-app.wasm 5 10

```

## Contributing

Contributions are welcome! We would like to hear it from you. For any questions or suggestions open an issue or start a discussion. For contributions please fork this repository and open a pull request with your changes.
