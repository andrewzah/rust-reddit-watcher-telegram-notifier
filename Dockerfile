
FROM rust:1-slim-buster as rust_builder

WORKDIR /build

COPY ./Cargo.toml Cargo.toml
COPY ./src/ ./src/

RUN apt-get -qq update -y \
  && apt-get -qq install -y \
    openssl \
    librust-openssl-dev \
  && rm -rf /var/lib/apt/lists/*

RUN cargo build --release \
  && mv ./target/release/sub_watcher sub_watcher \
  && rm -rf ./target/

FROM debian:stable-slim

RUN apt update \
  && apt install -y openssl ca-certificates

COPY --from=rust_builder /build/sub_watcher /sub_watcher

CMD ["/sub_watcher"]
