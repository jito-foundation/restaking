##### Builder image
FROM rust:1.80.0-slim-bullseye as builder

RUN apt-get update && apt-get install -y \
    libudev-dev \
    clang \
    pkg-config \
    libssl-dev \
    build-essential \
    cmake \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release --bin jito-vault-crank

##### Final image
FROM debian:bullseye-slim as cranker

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    procps \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/jito-vault-crank /usr/local/bin/jito-vault-crank

ENTRYPOINT ["jito-vault-crank"]
