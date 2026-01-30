# @anvanster/memrl

Memory-augmented reinforcement learning for Claude Code - persistent memory that learns from experience.

## Installation

```bash
npm install -g @anvanster/memrl
```

## Usage

After installation, you have access to two commands:

- `memrl` - CLI tool for managing memories
- `memrl-mcp` - MCP server for Claude Code integration

### Setup with Claude Code

```bash
claude mcp add memrl --scope user -- memrl-mcp
```

### CLI Commands

```bash
# Initialize MemRL
memrl init

# Capture an episode
memrl capture --prompt "Fixed the auth bug"

# Search memories
memrl retrieve "database issues"

# View statistics
memrl stats
```

## Documentation

See the [full documentation](https://github.com/anvanster/memrl) for more details.

## License

Apache-2.0
