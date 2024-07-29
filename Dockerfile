FROM rust:alpine3.20 as builder
WORKDIR /usr/src/app
COPY . .
RUN apk add --no-cache musl-dev
RUN cargo build --release

FROM alpine:3.20
RUN apk add --no-cache libgcc openssh
COPY --from=builder /usr/src/app/target/release/bazzite-updater /usr/local/bin/bazzite-updater
RUN mkdir /app
WORKDIR /app
CMD ["bazzite-updater"]
