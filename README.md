# Vitax

**Vita-eXtended** - A modern, safe, and cross-platform directory analysis tool written in Rust.

Vitax is an enhanced version of the original [vita](https://github.com/rxxuzi/vita) project, designed to recursively traverse directories and intelligently output file contents. Perfect for preparing project overviews for AI chat tools and code reviews.

## Installation

### Prerequisites

- Rust 1.89.0 or later

### Building from Source

```bash
git clone https://github.com/rxxuzi/vitax.git
cd vitax
cargo build --release
```

## Usage

```bash
# Analyze a single file
vitax file.txt

# Analyze entire directory
vitax /path/to/project
```

## License

MIT License