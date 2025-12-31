# Crack Hash

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-1.1.0-green.svg)](https://github.com/lemusic/hash-cracker/releases)
[![GitHub stars](https://img.shields.io/github/stars/kOaDT/crack-hash?style=social)](https://github.com/kOaDT/crack-hash)

A fast, multi-threaded hash cracking tool written in Rust. This tool performs dictionary attacks against hashed passwords.

![Crack Hash CLI](screen.png)

## Features

- **Single hash cracking**: Crack individual hashes
- **Batch TXT processing**: Crack multiple hashes from a text file (one hash per line)
- **Batch CSV processing**: Crack hashes from CSV files with customizable column mapping
- **Multi-threaded**: Leverages all CPU cores for faster cracking
- **Multiple algorithms**: Supports MD5, SHA1, and SHA256

## Usage (Local)

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)

### Build and run

```bash
cargo build --release
./target/release/crack-hash --help
```

Or directly with Cargo:

```bash
cargo run --release -- --help
```

---

## Usage (Docker)

If you prefer using Docker:

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

## Commands

> **Note:** All examples below use `cargo run --`. After building with `cargo build --release`, you can use `./target/release/crack-hash` directly.

### Single Hash Mode

Crack a single hash:

```bash
cargo run -- single -a <ALGORITHM> -H <HASH> -w <WORDLIST>
```

| Argument | Short | Description |
|----------|-------|-------------|
| `--algo` | `-a` | Hash algorithm (md5, sha1, sha256) |
| `--hash` | `-H` | Target hash to crack |
| `--wordlist` | `-w` | Path to the wordlist file |

**Example:**

```bash
cargo run -- single -a md5 -H 5d41402abc4b2a76b9719d911017c592 -w wordlist.txt
```

### Batch TXT Mode

Crack multiple hashes from a text file (one hash per line):

```bash
cargo run -- batch-txt -a <ALGORITHM> -i <INPUT_FILE> -o <OUTPUT_FILE> -w <WORDLIST>
```

| Argument | Short | Description |
|----------|-------|-------------|
| `--algo` | `-a` | Hash algorithm (md5, sha1, sha256) |
| `--input` | `-i` | Input file containing hashes (one per line) |
| `--output` | `-o` | Output file for results |
| `--wordlist` | `-w` | Path to the wordlist file |

**Example:**

```bash
cargo run -- batch-txt -a md5 -i hashes.txt -o results.txt -w wordlist.txt
```

### Batch CSV Mode

Crack hashes from a CSV file with customizable format:

```bash
cargo run -- batch-csv -a <ALGORITHM> -i <INPUT_CSV> -o <OUTPUT_CSV> -w <WORDLIST> [OPTIONS]
```

| Argument | Short | Description | Default |
|----------|-------|-------------|---------|
| `--algo` | `-a` | Hash algorithm (md5, sha1, sha256) | - |
| `--input` | `-i` | Input CSV file | - |
| `--output` | `-o` | Output CSV file | - |
| `--wordlist` | `-w` | Path to the wordlist file | - |
| `--hash-column` | `-c` | Column index containing the hash (0-based) | 0 |
| `--delimiter` | `-d` | CSV delimiter character | , |
| `--no-header` | - | Disable header row parsing (headers enabled by default) | false |

**Example with header:**

```bash
cargo run -- batch-csv -a sha256 -i users.csv -o cracked.csv -w wordlist.txt -c 2
```

**Example without header:**

```bash
cargo run -- batch-csv -a md5 -i data.csv -o results.csv -w wordlist.txt -c 1 --no-header
```

**Example with custom delimiter (semicolon):**

```bash
cargo run -- batch-csv -a md5 -i data.csv -o results.csv -w wordlist.txt -d ";"
```

## Algorithm Examples

- [MD5](/docs/md5.md)
- [SHA1](/docs/sha1.md)
- [SHA256](/docs/sha256.md)

## Contributing

Contributions are welcome. To contribute:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Commit your changes (`git commit -m 'Add your feature'`)
4. Push to the branch (`git push origin feature/your-feature`)
5. Open a Pull Request

Please ensure your code follows the existing style and includes appropriate tests.

## License

This project is licensed under the MIT License.

```
MIT License

Copyright (c) 2025

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
