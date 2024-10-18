FROM docker.io/library/rust:1.81.0 AS builder
RUN apt-get update && apt-get install -y musl-tools pkg-config libssl-dev build-essential
RUN mkdir /build
COPY . /build/
WORKDIR /build
RUN cargo build --release
RUN chmod +x /build/target/release/function


FROM docker.io/library/rust:1.81.0
COPY --from=builder /build/target/release/function /function
ENTRYPOINT ["/function"]