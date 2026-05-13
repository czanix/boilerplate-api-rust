FROM rust:1.77 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/czanix-api /api
EXPOSE 3000
ENTRYPOINT ["/api"]
