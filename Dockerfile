FROM rust:latest as builder

RUN apt update && apt-get install -y libasound2-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /libretakt
COPY src src
COPY server server
COPY common common
COPY fff-forward fff-forward
COPY uigraphics uigraphics
COPY mverb mverb
COPY examples examples
COPY Cargo.toml .
COPY Cargo.lock .

RUN cargo build --release --examples && cargo build --release -p server


FROM debian:buster-slim
RUN apt update && apt-get install -y ffmpeg && rm -rf /var/lib/apt/lists/*
WORKDIR /libretakt
COPY samples samples
COPY --from=builder /libretakt/target/release/server /usr/local/bin/
COPY --from=builder /libretakt/target/release/examples/headless_ffmpeg_client /usr/local/bin/
CMD ["server"]

