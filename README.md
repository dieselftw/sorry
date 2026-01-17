# sorry

A tiny Rust CLI that sends your terminal/git mistakes to an LLM and prints helpful advice.

```bash
sorry I accidentally force pushed to main
```

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
# Binary at ./target/release/sorry
```

## Setup

Configure with your preferred provider:

```bash
# OpenAI
sorry --config-openai sk-your-api-key-here

# Groq (free tier available!)
sorry --config-groq gsk-your-api-key-here
```

## Usage

Just type `sorry` followed by your problem:

```bash
sorry I made a mistake with the git commit
sorry I deleted a file I shouldn't have
sorry I ran rm -rf on the wrong directory
sorry what does git reflog do
sorry how do I undo my last commit but keep the changes
```

## Commands

| Command | Description |
|---------|-------------|
| `sorry <message>` | Send your problem to the LLM and get help |
| `sorry --config-openai <key>` | Configure OpenAI API key (sets as active provider) |
| `sorry --config-groq <key>` | Configure Groq API key (sets as active provider) |
| `sorry --show-config` | Show active provider and settings (keys hidden) |
| `sorry --help` | Show help |
| `sorry --version` | Show version |

## Config File

Config is stored at:
- **macOS**: `~/Library/Application Support/sorry/config.json`
- **Linux**: `~/.config/sorry/config.json`
- **Windows**: `%APPDATA%\sorry\config.json`

Example config:

```json
{
  "provider": "openai",
  "providers": {
    "openai": {
      "api_key": "sk-...",
      "base_url": "https://api.openai.com/v1",
      "model": "gpt-4.1-mini"
    },
    "groq": {
      "api_key": "gsk-...",
      "base_url": "https://api.groq.com/openai/v1",
      "model": "llama-3.1-70b-versatile"
    }
  }
}
```

You can manually edit this file to:
- Change models (e.g., `gpt-4o`, `llama-3.3-70b-versatile`)
- Use a different OpenAI-compatible API (change `base_url`)
- Switch providers by changing `"provider"`

## How It Works

1. Parses your message from the command line
2. Loads your config to get the API key and endpoint
3. Sends a chat completion request to an OpenAI-compatible API
4. Prints the response

That's it. It's read-only and never executes any commands or modifies your system.

## License

MIT
