FROM rust:1 as builder

WORKDIR /app

COPY . /app
RUN cargo build --release


# To run locally with podman:
# podman run --security-opt label=disable --device /dev/fuse --cap-add=CAP_NET_ADMIN --env-file .env ferrisbot:latestp
FROM quay.io/containers/podman:latest

# Need this environment variable to tell ferris-bot it's inside a container
# Not all features are supported within the container
ENV IS_RUNNING_IN_CONTAINER="true"

COPY --from=builder /app/target/release/ferris-bot /app/ferris-bot

USER podman

ENTRYPOINT ["/app/ferris-bot"]
