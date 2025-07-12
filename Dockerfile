FROM rust:1-alpine AS builder

RUN apk update && \
    apk add libressl-dev musl-dev pkgconfig

WORKDIR /app

COPY . .

RUN cargo build --all --release

FROM alpine:latest AS release

RUN apk update && \
    apk add ffmpeg

COPY --from=builder /app/target/release/api /usr/bin/transcoder-api
COPY --from=builder /app/target/release/cli /usr/bin/transcoder

CMD ["transcoder-api"]