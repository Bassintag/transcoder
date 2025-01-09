FROM golang:alpine AS builder

WORKDIR /app

COPY src/go.mod src/go.sum ./

RUN go mod download

COPY src/*.go ./

RUN CGO_ENABLED=0 GOOS=linux go build -o /a.out .

FROM alpine:latest AS runner

RUN mkdir /data

RUN apk update &&\
    apk upgrade &&\
    apk add --no-cache ffmpeg

WORKDIR /app

COPY --from=builder /a.out ./a.out

ENV ROOT_FOLDER=/data
ENV GIN_MODE=release

CMD ./a.out

