#############################
# Build stage
#############################
FROM rust:1.87 AS builder
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
COPY ./src ./relayer-order-api/src
COPY ./Cargo.toml ./relayer-order-api/Cargo.toml

WORKDIR /workspace/relayer-order-api
RUN cargo build --release


#############################
# Runtime stage
#############################
FROM debian:bookworm-slim AS runtime

# Install CA certificates so HTTPS works
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create an unprivileged user to run the binary
RUN useradd -m relayer

# Set work directory to user home; this is where the JSON will be written
WORKDIR /home/relayer

COPY --from=builder /workspace/relayer-order-api/target/release/main /usr/local/bin/relayer-order-api

# Switch to the non-root user
USER relayer

COPY ./.env ./.env
EXPOSE 3032
ENTRYPOINT ["relayer-order-api"]
