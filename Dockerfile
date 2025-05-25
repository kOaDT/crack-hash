FROM rust:1.75
WORKDIR /app
RUN apt-get update && apt-get install -y \
    vim \
    && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build
CMD ["cargo", "run", "--", "--help"] 