FROM rust:latest as builder

RUN apt update && apt-get install -y libasound2-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /libretakt
COPY src src
COPY common common
COPY fff-forward fff-forward
COPY uigraphics uigraphics
COPY mverb mverb
COPY examples examples
COPY Cargo.toml .
COPY Cargo.lock .

RUN cargo build --release --examples && cargo build --release
