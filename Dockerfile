FROM rust:1 AS builder

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

COPY . .

RUN cargo build --all --release --target x86_64-unknown-linux-musl

CMD ls -l target/release

FROM alpine:latest AS release

WORKDIR /app

COPY --from=builder /app/target/release/api .
COPY --from=builder /app/target/release/cli .

CMD ["./api"]