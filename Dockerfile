FROM rust:1 as builder

WORKDIR /app

COPY . /app
RUN cargo build --release


FROM quay.io/containers/podman:latest

# Need this environment variable to tell ferris-bot it's inside a container
# Not all features are supported within the container
ENV IS_RUNNING_IN_CONTAINER="true"

COPY --from=builder /app/target/release/ferris-bot /app/ferris-bot

ENTRYPOINT ["/app/ferris-bot"]
