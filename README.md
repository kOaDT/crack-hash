# Hash Cracker

A hash cracking tool.

## Usage with Docker

### 1. Build the container
```bash
docker compose build
```

### 2. Launch the container in development mode
```bash
docker compose up -d
docker compose exec hash-cracker bash
```

### 3. Compile and run
```bash
# Inside the container
cargo build
cargo run -- --help
```

### 4. Usage example
```bash
# Create a sample wordlist file
echo -e "password\n123456\nadmin\ntest" > wordlist.txt

# Run the hash cracker
cargo run -- --algo md5 --hash 5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8 --wordlist wordlist.txt
```

## CLI Arguments

- `--algo` or `-a`: Hash algorithm (md5, sha1, sha256, sha512)
- `--hash` or `-H`: Target hash to crack
- `--wordlist` or `-w`: Path to the wordlist file

## Project Structure

```
hash-cracker/
├── src/
│   └── main.rs          # Main entry point with CLI
├── Cargo.toml           # Rust project configuration
├── Dockerfile           # Docker image for the environment
├── docker-compose.yml   # Docker Compose configuration
└── README.md           # This file
``` 