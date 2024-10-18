# Build stage
FROM docker.io/library/rust:1.81.0 AS builder

# Install musl tools and dependencies
RUN apt-get update && apt-get install -y musl-tools pkg-config libssl-dev build-essential

# Set environment variables for cross-compiling with musl
#ENV CC=musl-gcc \
#    AR=ar \
#    RANLIB=ranlib \
#    CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc \
#    PKG_CONFIG_ALLOW_CROSS=1

# Set RUSTFLAGS to ensure static linking
#ENV RUSTFLAGS="-C target-feature=+crt-static"

# Create build directory
RUN mkdir /build

# Copy the source code into the container
COPY . /build/

# Set the working directory
WORKDIR /build

# Add musl target for static linking and build the project in release mode
#RUN rustup target add x86_64-unknown-linux-musl && \
#RUN cargo build --release --target x86_64-unknown-linux-musl

RUN cargo build --release

# Ensure the binary has executable permissions
RUN chmod +x /build/target/release/http-client

# Production stage using scratch (minimalist image)
FROM docker.io/library/rust:1.81.0

# Copy the compiled binary from the build stage
COPY --from=builder /build/target/release/http-client /http-client

# Set the entrypoint to execute the binary with an argument
ENTRYPOINT ["/http-client", "http://10.152.183.42:80"]