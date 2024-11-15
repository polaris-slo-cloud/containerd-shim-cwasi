# Build stage
FROM docker.io/library/rust:1.81.0 AS builder

# Install musl tools and dependencies
RUN apt-get update && apt-get install -y musl-tools pkg-config libssl-dev build-essential

# Create build directory
RUN mkdir /build

# Copy the source code into the container
COPY . /build/

# Set the working directory
WORKDIR /build


RUN cargo build --release

# Ensure the binary has executable permissions
RUN chmod +x /build/target/release/fanout-native
# Production stage using scratch (minimalist image)
FROM docker.io/library/rust:1.81.0
# Copy the compiled binary from the build stage
COPY --from=builder /build/target/release/fanout-native /fanout-native
# Set the entrypoint to execute the binary with an argument
ENTRYPOINT ["/fanout-native"]