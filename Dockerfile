FROM rust:latest AS builder

RUN rustup target add wasm32-unknown-unknown \
    && cargo install trunk

WORKDIR /app
COPY . .

RUN make build-release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/client /app/client

EXPOSE 3000
CMD ["/app/client"]
