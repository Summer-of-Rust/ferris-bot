# Ferris-Bot

## Development

You will need a discord bot token with the appropriate scopes and permissions to use slash commands. Create a `.env` file with the following content:
```
DISCORD_TOKEN=YOUR_TOKEN_HERE
```

Ferris-Bot supports running in 2 different modes
- **Run directly on host**: the bot will run directly on your host as a regular process. Your host needs [podman](https://podman.io/) installed as the bot invokes podman to run containers for executing code. To run Ferris-Bot this way, simply use `cargo run`
- **Run as a container**: the bot will run as a container and spawn nested containers for executing code. Running this way is a little more work:

  First, to build the image locally (this will compile the bot in release mode, may take a long time):
  ```bash
  podman build -t ferrisbot:latest .
  ```

  After the container is built, it can be ran using rootless podman
  ```
  podman run --rm --security-opt label=disable --device /dev/fuse --env-file .env ferrisbot:latest
  ```