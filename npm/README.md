# @anvanster/tempera

Tempera - persistent memory for Claude Code that learns from experience.

## Installation

```bash
npm install -g @anvanster/tempera
```

## Usage

After installation, you have access to two commands:

- `tempera` - CLI tool for managing memories
- `tempera-mcp` - MCP server for Claude Code integration

### Setup with Claude Code

```bash
claude mcp add tempera --scope user -- tempera-mcp
```

### CLI Commands

```bash
# Initialize Tempera
tempera init

# Capture an episode
tempera capture --prompt "Fixed the auth bug"

# Search memories
tempera retrieve "database issues"

# View statistics
tempera stats
```

## Documentation

See the [full documentation](https://github.com/anvanster/tempera) for more details.

## License

Apache-2.0
