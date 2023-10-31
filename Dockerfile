FROM rust:1.70 as build-env
WORKDIR /app
COPY . /app
RUN apt-get update && apt-get install -y protobuf-compiler
RUN cargo build --release

FROM debian:buster-slim
COPY target/release/otel-tokio-axum  /
CMD ["./otel-tokio-axum"]