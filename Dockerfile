FROM rust:latest AS builder

RUN apt-get update && apt-get install -y libasound2-dev ca-certificates wget && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown

RUN wget -qO- https://github.com/thedodd/trunk/releases/download/v0.20.2/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C /usr/local/bin trunk

WORKDIR /app
COPY . .

RUN trunk build --release && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/client /app/client

EXPOSE 3000
CMD ["/app/client"]
