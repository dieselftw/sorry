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

Run the interactive configuration for your preferred provider:

```bash
# OpenAI
sorry --config-openai

# Groq (free tier available!)
sorry --config-groq
```

You'll be prompted for:
1. **API key** - paste your key
2. **Model selection** - pick from a list or press Enter for the default

Example:
```
🔧 Configuring groq

Enter API key: gsk-xxxxx

Available models:
  1. llama-3.3-70b-versatile (default)
  2. llama-3.1-8b-instant
  3. llama3-70b-8192
  4. llama3-8b-8192
  5. mixtral-8x7b-32768
  6. gemma2-9b-it

Select model [1-6] or press Enter for default: 

✓ Configured groq with model 'llama-3.3-70b-versatile'
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
| `sorry --config-openai` | Interactive OpenAI setup (API key + model) |
| `sorry --config-groq` | Interactive Groq setup (API key + model) |
| `sorry --show-config` | Show active provider and settings (keys hidden) |
| `sorry --help` | Show help |
| `sorry --version` | Show version |

## Available Models

### OpenAI
- `gpt-4.1-mini` (default)
- `gpt-4.1-nano`
- `gpt-4.1`
- `gpt-4o`
- `gpt-4o-mini`
- `o1`, `o1-mini`, `o3-mini`

### Groq
- `llama-3.3-70b-versatile` (default)
- `llama-3.1-8b-instant`
- `llama3-70b-8192`
- `llama3-8b-8192`
- `mixtral-8x7b-32768`
- `gemma2-9b-it`

## Config File

Config is stored at:
- **macOS**: `~/Library/Application Support/sorry/config.json`
- **Linux**: `~/.config/sorry/config.json`
- **Windows**: `%APPDATA%\sorry\config.json`

You can manually edit this file to use custom models or OpenAI-compatible APIs.

## How It Works

1. Parses your message from the command line
2. Loads your config to get the API key and endpoint
3. Sends a chat completion request to an OpenAI-compatible API
4. Prints the response

That's it. It's read-only and never executes any commands or modifies your system.

## License

MIT
