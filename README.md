# Hacker News MCP

This is a Model Context Protocol (MCP) server that provides tools for accessing Hacker News data. It enables LLMs to search and retrieve content from Hacker News through a standardized interface.

## Features

- **hn_web_search**: Perform web searches for Hacker News content
- **hn_news_search**: Search for news articles on Hacker News with filtering options
- **hn_local_search**: Find Hacker News discussions related to specific locations

## Prerequisites

You need a HN API key to use this server. You can obtain one by visiting the Hacker News API documentation.

## Installation

```bash
git clone https://github.com/tacogips/hn-mcp.git
cd hn-mcp
cargo build --release
```

## Running the Server

There are two ways to provide your HN API key:

1. Set it as an environment variable:
   ```bash
   export HN_API_KEY=your_api_key_here
   ```

2. Provide it directly as a command-line argument:
   ```bash
   cargo run --bin hn-mcp --api-key your_api_key_here stdio
   ```

Choose the mode that suits your needs:

### STDIN/STDOUT Mode

This mode is useful when you want to pipe data directly to and from the server:

```bash
# Run in STDIN/STDOUT mode with environment variable
cargo run --bin hn-mcp --api-key $HN_API_KEY stdio

# Run in STDIN/STDOUT mode with debug logging
cargo run --bin hn-mcp --api-key your_api_key_here stdio --debug
```

### HTTP Mode

HTTP mode runs an HTTP server with Server-Sent Events (SSE):

```bash
# Run in HTTP mode (default address: 0.0.0.0:3000)
cargo run --bin hn-mcp --api-key $HN_API_KEY http

# Run in HTTP mode with custom address
cargo run --bin hn-mcp --api-key your_api_key_here http --address 127.0.0.1:8080

# Run in HTTP mode with debug logging
cargo run --bin hn-mcp --api-key your_api_key_here http --debug
```

## Command-Line Options

The server supports the following command-line options:

```
USAGE:
    hn-mcp [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -a, --api-key <API_KEY>    HN API key, required if HN_API_KEY environment variable is not set
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
# If you've set the HN_API_KEY environment variable:
cargo run --example client

# Or, set it when running the example:
HN_API_KEY=your_api_key_here cargo run --example client
```

The example client demonstrates:

1. STDIN/STDOUT communication with the server
2. HTTP/SSE communication
3. Making web search, news search, and local search requests
4. Handling and parsing of various response formats

## Available Tools

The server provides the following tools:

### 1. `hn_web_search`

Performs web searches using the HN Search API.

Parameters:

- `query` (required): The search query (max 400 chars, 50 words)
- `count` (optional): Number of results to return (1-20, default 10)
- `offset` (optional): Pagination offset (max 9, default 0)

Example:

```json
{
  "name": "hn_web_search",
  "arguments": {
    "query": "What is Rust programming?",
    "count": 5
  }
}
```

### 2. `hn_news_search`

Searches for news articles using the HN News Search API.

Parameters:

- `query` (required): News search query (max 400 chars, 50 words)
- `count` (optional): Number of results (1-50, default 20)
- `offset` (optional): Pagination offset (max 9, default 0)
- `country` (optional): Country code (default US)
  - Available options: ALL, AR, AU, AT, BE, BR, CA, CL, DK, FI, FR, DE, HK, IN, ID, IT, JP, KR, MY, MX, NL, NZ, NO, CN, PL, PT, PH, RU, SA, ZA, ES, SE, CH, TW, TR, GB, US
- `search_lang` (optional): Search language (default en)
  - Available options: ar, eu, bn, bg, ca, zh-hans, zh-hant, hr, cs, da, nl, en, en-gb, et, fi, fr, gl, de, gu, he, hi, hu, is, it, ja, kn, ko, lv, lt, ms, ml, mr, nb, pl, pt, pt-br, pa, ro, ru, sr, sk, sl, es, sv, ta, te, th, tr, uk, vi
- `freshness` (optional): Timeframe filter (h for hour, d for day, w for week, m for month, y for year)

Example:

```json
{
  "name": "hn_news_search",
  "arguments": {
    "query": "AI advancements",
    "count": 5,
    "country": "JP",
    "search_lang": "en",
    "freshness": "w"
  }
}
```

### 3. `hn_local_search`

Searches for Hacker News content related to specific locations.

Parameters:

- `query` (required): The local search query (e.g., "startups in San Francisco")
- `count` (optional): Number of results to return (1-20, default 5)

Example:

```json
{
  "name": "hn_local_search",
  "arguments": {
    "query": "Tech companies in Seattle",
    "count": 3
  }
}
```

## Implementation Notes

- The server implements rate limiting to adhere to API restrictions
- Local search automatically falls back to web search if no local results are found
- Results include detailed information where available
- News search supports comprehensive filtering by country, language, and freshness
- All tools properly handle API errors and rate limiting with appropriate user feedback
- API key validation occurs at startup to ensure proper configuration

## Rate Limits

The API has the following rate limits:

- 1 request per second
- 15,000 requests per month

The server implements these rate limits to prevent exceeding the API quotas.

## MCP Protocol Integration

This server implements the Model Context Protocol (MCP) which allows it to be easily integrated with LLM clients that support the protocol. For more information about MCP, visit [the MCP repository](https://github.com/modelcontextprotocol/mcp).

## License

MIT License