FROM docker.io/library/rust:1.80.0-slim-bookworm AS builder
RUN apt-get update && apt-get install -y musl-tools
ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN mkdir /build
COPY . /build/
WORKDIR /build
RUN rustup target add x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl
RUN chmod +x /build/target/x86_64-unknown-linux-musl/release/http-server

FROM scratch
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/http-server /http-server
COPY --from=builder /build/file_100M.txt /file_100M.txt
ENTRYPOINT ["/http-server","file_100M.txt"]
