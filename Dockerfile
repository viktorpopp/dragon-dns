#
# Layer we will re-use for the planner and builder
#
FROM rust:1.97.1-slim-trixie AS chef

WORKDIR /app

RUN cargo install cargo-chef --locked

#
# Prepare to build dependencies
#
FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

#
# Build the dependencies and application
#
FROM chef AS builder
RUN apt-get update -y &&                        \
    apt-get install pkg-config libssl-dev -y

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --locked

#
# Download
#
FROM debian:trixie-slim AS runtime
RUN apt-get update &&                                           \
    apt-get install -y ca-certificates &&                       \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/dragon-dns /bin/ddns
ENTRYPOINT ["/bin/ddns"]
