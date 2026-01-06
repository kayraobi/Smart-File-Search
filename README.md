# Smart File Search

CLI tool for semantic file search using vector embeddings. Search files by meaning, not just keywords.

## Installation

```bash
cargo build --release
./target/release/smart_file_search
```

## Usage

```bash
smart_file_search <command> [options]
```

## Commands

| Command | Description | Options |
|---------|-------------|---------|
| `update-db` | Index files in database | `-p <path>` - Path to index (default: current directory) |
| `search <query>` | Search indexed files (one-shot) | - |
| `interactive-search` | Interactive search mode (REPL) | - |
| `info` | Show database statistics | - |

## Examples

```bash
# Index files in current directory
smart_file_search update-db

# Index files in specific path
smart_file_search update-db -p /home/user/Downloads

# Search for files
smart_file_search search "database schema"

# Interactive search mode
smart_file_search interactive-search

# View database info
smart_file_search info
```

## Command Details

**update-db** [-p *path*]

Recursively scans the specified path and indexes all non-hidden files. Hidden files (starting with `.`) are excluded. The database is cleared and rebuilt each time this command runs.

**search** *query*

Performs a one-shot semantic search for *query*. Results are output as JSONL to stdout.

**interactive-search**

Starts interactive search mode. Reads queries from stdin, outputs JSONL results to stdout. All informational messages go to stderr. Press Ctrl+C to exit.

**info**

Displays current database statistics including number of files, database size, and last update time.

## Output Format

Search commands output JSONL to stdout:

```jsonl
{"search":"test","results":[{"file_name":"test.txt","path":"/home/user/test.txt","distance":0.95}]}
```

- `distance`: Cosine distance (0 = unrelated, 1 = identical, 2 = opposite)
- `results`: Sorted by relevance (lowest distance first)
- `path`: Absolute file path

