FROM rust:1.82-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin trustverify

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/trustverify /usr/local/bin/
COPY --from=builder /app/signed_registry.json /app/signed_registry.json
COPY --from=builder /app/samples /app/samples
CMD ["trustverify", "serve", "--host", "0.0.0.0", "--port", "8080", "--registry", "/app/signed_registry.json"]
