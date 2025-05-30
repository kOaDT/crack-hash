# Crack Hash

A hash cracking tool.

## Usage with Docker

### 1. Build the container
```bash
docker compose build
```

### 2. Launch the container
```bash
docker compose up -d
docker compose exec crack-hash bash
```

### 3. Compile and run
```bash
# Inside the container
cargo build
cargo run -- --help
```

### 4. Usage example

- [MD5](/docs/md5.md)
- [SHA1](/docs/sha1.md)
- [SHA256](/docs/sha256.md)

## CLI Arguments

- `--algo` or `-a`: Hash algorithm (md5, sha1, sha256, sha512)
- `--hash` or `-H`: Target hash to crack
- `--wordlist` or `-w`: Path to the wordlist file