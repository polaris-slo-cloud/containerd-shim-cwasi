# "CWASI" Containerd shim

WIP: Should be able to spawn containers and wasm modules

Based: https://github.com/keniack/runwasi/tree/main/crates/containerd-shim-wasmedge

Build
```
cargo build --release
```

Copy binary to $PATH
```
sudo cp target/release/containerd-shim-cwasi-v1 /usr/local/bin/containerd-shim-cwasi-v1
```

Usage
```
sudo crictl pull docker.io/wasmedge/example-wasi:latest
sudo ctr -n k8s.io run  --runtime=io.containerd.cwasi.v1 docker.io/wasmedge/example-wasi:latest cwasi /wasi_example_main.wasm 50000000
```