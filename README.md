# sorry

A tiny Rust CLI that sends your terminal/git mistakes to an LLM and prints helpful advice.

It automatically includes your last 10 terminal commands for context.

```bash
sorry I messed up
```

## Installation

```bash
cargo install --path .
```

## Setup

### 1. Configure a provider

```bash
# OpenAI
sorry --config-openai

# Groq (free tier available!)
sorry --config-groq
```

You'll be prompted for your API key and model:

```
🔧 Configuring groq

Enter API key: gsk-xxxxx
Enter model name (openai/gpt-oss-20b): 

✓ Configured groq with model 'openai/gpt-oss-20b'
```

### 2. Choose your mood

```bash
sorry --behaviour
```

```
🎭 Configure sorry's behaviour

Choose a mood:

  1. Treat me like a princess
  2. Treat me like a bro
  3. Treat me like a bitch

Select mood [1-3]: 
```

**Moods:**
- **Princess** 👸 - Gentle, supportive, encouraging
- **Bro** 🤙 - Casual, chill, brief
- **Bitch** 💅 - Roasts you mercilessly, but still helps

## Usage

```bash
sorry I made a mistake
sorry what did I just do
sorry help
```

It automatically reads your last 10 commands from shell history (zsh/bash) and includes them in the prompt, so the LLM knows what you did.

## Commands

| Command | Description |
|---------|-------------|
| `sorry <message>` | Get help (includes last 10 commands as context) |
| `sorry --config-openai` | Configure OpenAI |
| `sorry --config-groq` | Configure Groq |
| `sorry --behaviour` | Choose your mood |
| `sorry --show-config` | Show current settings |

## Project Structure

```
src/
├── main.rs     # CLI entry point
├── config.rs   # Config types, moods, file I/O
├── cli.rs      # Interactive configuration
├── api.rs      # LLM API calls
└── history.rs  # Shell history reading
```

## License

MIT
