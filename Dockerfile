FROM rust:1.75

WORKDIR /app

# Install dependencies and cargo-watch for hot reloading
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    cargo install cargo-watch@8.4.0 --locked

# Copy the config files
COPY config/default.json config/development.json /app/config/

# Expose the API port
EXPOSE 8080

# Run with cargo-watch for hot reloading
CMD ["cargo", "watch", "-x", "run", "-w", "src", "-w", "Cargo.toml"]