FROM rust:1.70 as build-env
WORKDIR /app
COPY . /app
RUN apt-get update && apt-get install -y protobuf-compiler libc6
RUN cargo build --release

FROM debian:bullseye-slim
RUN mkdir config
COPY --from=build-env /app/config/default.toml /config
COPY --from=build-env /app/config/development.toml /config
COPY --from=build-env /app/target/release/otel-tokio-axum  /
CMD ["./otel-tokio-axum"]