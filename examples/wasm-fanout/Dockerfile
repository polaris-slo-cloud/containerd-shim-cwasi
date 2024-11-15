FROM docker.io/library/rust:1.80.0-slim-bookworm AS builder
RUN apt-get update && \
    apt-get install -y clang && \
    apt-get clean
RUN rustup target add wasm32-wasi
ENV RUSTFLAGS="--cfg wasmedge --cfg tokio_unstable"
RUN mkdir /build
COPY . /build/
RUN ls -lah /build
WORKDIR /build
RUN cargo build --target wasm32-wasi --release
RUN ls -lah
RUN chmod +x target/wasm32-wasi/release/wasm-fanout.wasm

FROM scratch
#ADD target/wasm32-wasi/release/func_a.wasm /
COPY --from=builder /build/target/wasm32-wasi/release/wasm-fanout.wasm /
ENTRYPOINT ["/wasm-fanout.wasm"]
