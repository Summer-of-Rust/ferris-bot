FROM rust:1 as builder

WORKDIR /app

COPY . /app
RUN cargo build --release


FROM debian:buster-slim

COPY --from=builder /app/target/release/ferris-bot /app/ferris-bot

ENTRYPOINT ["/app/ferris-bot"]
