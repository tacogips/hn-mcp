# Hacker News MCP

This is a Model Context Protocol (MCP) server that provides tools for accessing Hacker News data. It enables LLMs to retrieve content from Hacker News through a standardized interface.

## Features

- **hn_top_stories**: Retrieves the top stories from Hacker News
- **hn_latest_stories**: Retrieves the latest stories from Hacker News
- **hn_best_stories**: Retrieves the best stories from Hacker News
- **hn_ask_stories**: Retrieves Ask HN stories from Hacker News
- **hn_show_stories**: Retrieves Show HN stories from Hacker News
- **hn_story_by_id**: Retrieves story details by ID from Hacker News

## Installation

```bash
git clone https://github.com/your-username/hn-mcp.git
cd hn-mcp
cargo build --release
```

## Running the Server

The server can be run in different modes:

### STDIN/STDOUT Mode

This mode is useful when you want to pipe data directly to and from the server:

```bash
# Run in STDIN/STDOUT mode
cargo run stdio

# Run in STDIN/STDOUT mode with debug logging
cargo run stdio --debug
```

### HTTP Mode

HTTP mode runs an HTTP server with Server-Sent Events (SSE):

```bash
# Run in HTTP mode (default address: 0.0.0.0:3000)
cargo run http

# Run in HTTP mode with custom address
cargo run http --address 127.0.0.1:8080

# Run in HTTP mode with debug logging
cargo run http --debug
```

## Command-Line Options

The server supports the following command-line options:

```
USAGE:
    hn-mcp [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help                 Print help information
    -v, --version              Print version information

SUBCOMMANDS:
    help     Print this message or the help of the given subcommand(s)
    http     Run the HN MCP server over HTTP with SSE
    stdio    Run the HN MCP server over stdio
```

For the `http` subcommand, you can specify the address and enable debug logging:

```
USAGE:
    hn-mcp http [OPTIONS]

OPTIONS:
    -a, --address <ADDRESS>    Address to use for HTTP server [default: 0.0.0.0:3000]
    -d, --debug                Enable debug logging
    -h, --help                 Print help information
```

For the `stdio` subcommand, you can enable debug logging:

```
USAGE:
    hn-mcp stdio [OPTIONS]

OPTIONS:
    -d, --debug                Enable debug logging
    -h, --help                 Print help information
```

## Using the Example Client

An example client is included to demonstrate how to interact with the server:

```bash
cargo run --example client
```

The example client demonstrates:

1. STDIN/STDOUT communication with the server
2. HTTP/SSE communication
3. Making requests for top stories, latest stories, best stories, etc.
4. Handling and parsing of story responses

## Available Tools

The server provides the following tools:

### 1. `hn_top_stories`

Retrieves the top stories from Hacker News.

Parameters:

- `limit` (optional): Number of stories to return (default: 10, max: 500)
- `chunk_size` (optional): Number of stories to fetch concurrently (default: 5, range: 1-10)

Example:

```json
{
  "name": "hn_top_stories",
  "arguments": {
    "limit": 5,
    "chunk_size": 3
  }
}
```

### 2. `hn_latest_stories`

Retrieves the latest stories from Hacker News.

Parameters:

- `limit` (optional): Number of stories to return (default: 10, max: 500)
- `chunk_size` (optional): Number of stories to fetch concurrently (default: 5, range: 1-10)

Example:

```json
{
  "name": "hn_latest_stories",
  "arguments": {
    "limit": 5
  }
}
```

### 3. `hn_best_stories`

Retrieves the best stories from Hacker News.

Parameters:

- `limit` (optional): Number of stories to return (default: 10, max: 500)
- `chunk_size` (optional): Number of stories to fetch concurrently (default: 5, range: 1-10)

Example:

```json
{
  "name": "hn_best_stories",
  "arguments": {
    "limit": 5
  }
}
```

### 4. `hn_ask_stories`

Retrieves Ask HN stories from Hacker News.

Parameters:

- `limit` (optional): Number of stories to return (default: 10, max: 500)
- `chunk_size` (optional): Number of stories to fetch concurrently (default: 5, range: 1-10)

Example:

```json
{
  "name": "hn_ask_stories",
  "arguments": {
    "limit": 5
  }
}
```

### 5. `hn_show_stories`

Retrieves Show HN stories from Hacker News.

Parameters:

- `limit` (optional): Number of stories to return (default: 10, max: 500)
- `chunk_size` (optional): Number of stories to fetch concurrently (default: 5, range: 1-10)

Example:

```json
{
  "name": "hn_show_stories",
  "arguments": {
    "limit": 5
  }
}
```

### 6. `hn_story_by_id`

Retrieves story details by ID from Hacker News.

Parameters:

- `id` (required): The Hacker News story ID

Example:

```json
{
  "name": "hn_story_by_id",
  "arguments": {
    "id": 12345
  }
}
```

## Implementation Notes

- Concurrent processing of story IDs for better performance
- LRU caching to reduce API calls for frequently requested stories
- Results include detailed story information where available
- All tools properly handle API errors with appropriate user feedback
- Stories are processed in configurable chunks (default: 5, max: 10) to optimize throughput

## MCP Protocol Integration

This server implements the Model Context Protocol (MCP) which allows it to be easily integrated with LLM clients that support the protocol. For more information about MCP, visit [the MCP repository](https://github.com/modelcontextprotocol/mcp).

## License

MIT License