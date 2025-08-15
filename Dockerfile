FROM rust:1.80-slim
WORKDIR /app
RUN apt-get update && apt-get install -y \
    vim \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml .
COPY src/ ./src/
RUN cargo build
CMD ["cargo", "run", "--", "--help"] 