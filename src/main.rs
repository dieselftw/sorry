use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process;

// ============================================================================
// Config types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct ProviderConfig {
    api_key: String,
    base_url: String,
    model: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Config {
    provider: Option<String>,
    providers: HashMap<String, ProviderConfig>,
}

impl Config {
    fn default_providers() -> HashMap<String, ProviderConfig> {
        let mut providers = HashMap::new();
        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                api_key: String::new(),
                base_url: "https://api.openai.com/v1".to_string(),
                model: "gpt-4.1-mini".to_string(),
            },
        );
        providers.insert(
            "groq".to_string(),
            ProviderConfig {
                api_key: String::new(),
                base_url: "https://api.groq.com/openai/v1".to_string(),
                model: "llama-3.1-70b-versatile".to_string(),
            },
        );
        providers
    }
}

// ============================================================================
// OpenAI-compatible API types
// ============================================================================

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    error: ApiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    message: String,
}

// ============================================================================
// CLI definition
// ============================================================================

#[derive(Parser, Debug)]
#[command(name = "sorry")]
#[command(about = "Send your mistakes to an LLM and get help")]
#[command(version)]
struct Args {
    /// Configure OpenAI API key
    #[arg(long = "config-openai", value_name = "API_KEY")]
    config_openai: Option<String>,

    /// Configure Groq API key
    #[arg(long = "config-groq", value_name = "API_KEY")]
    config_groq: Option<String>,

    /// Show current configuration (without revealing keys)
    #[arg(long = "show-config")]
    show_config: bool,

    /// The prompt to send to the LLM
    #[arg(trailing_var_arg = true)]
    prompt: Vec<String>,
}

// ============================================================================
// Config file helpers
// ============================================================================

fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sorry");
    config_dir.join("config.json")
}

fn load_config() -> Config {
    let path = get_config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    }
}

fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content)?;
    Ok(())
}

fn configure_provider(provider: &str, api_key: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = load_config();

    // Ensure we have default provider configs
    if config.providers.is_empty() {
        config.providers = Config::default_providers();
    }

    // Update the API key for this provider
    if let Some(provider_config) = config.providers.get_mut(provider) {
        provider_config.api_key = api_key;
    } else {
        // Add provider with defaults if it doesn't exist
        let defaults = Config::default_providers();
        if let Some(default_config) = defaults.get(provider) {
            config.providers.insert(
                provider.to_string(),
                ProviderConfig {
                    api_key,
                    base_url: default_config.base_url.clone(),
                    model: default_config.model.clone(),
                },
            );
        }
    }

    // Set as active provider
    config.provider = Some(provider.to_string());

    save_config(&config)?;
    println!("✓ Configured {} as the active provider.", provider);
    Ok(())
}

fn show_config() {
    let config = load_config();

    match &config.provider {
        Some(provider) => {
            println!("Active provider: {}", provider);
            if let Some(pc) = config.providers.get(provider) {
                println!("  Base URL: {}", pc.base_url);
                println!("  Model: {}", pc.model);
                let key_status = if pc.api_key.is_empty() {
                    "not set"
                } else {
                    "configured (hidden)"
                };
                println!("  API Key: {}", key_status);
            }
        }
        None => {
            println!("No provider configured.");
            println!("Run 'sorry --config-openai <api-key>' or 'sorry --config-groq <api-key>' to set up.");
        }
    }
}

// ============================================================================
// LLM API call
// ============================================================================

fn call_llm(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let config = load_config();

    let provider_name = config.provider.ok_or(
        "No provider configured. Run 'sorry --config-openai <api-key>' or 'sorry --config-groq <api-key>' first."
    )?;

    let provider = config.providers.get(&provider_name).ok_or(format!(
        "Provider '{}' not found in config.",
        provider_name
    ))?;

    if provider.api_key.is_empty() {
        return Err(format!(
            "API key not set for provider '{}'. Run 'sorry --config-{} <api-key>' to configure.",
            provider_name, provider_name
        )
        .into());
    }

    let url = format!("{}/chat/completions", provider.base_url);

    let request_body = ChatRequest {
        model: provider.model.clone(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant that helps developers undo or understand mistakes they made in the terminal or with git. Be concise and practical. When suggesting commands, explain what they do.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ],
    };

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", provider.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()?;

    let status = response.status();
    let body = response.text()?;

    if !status.is_success() {
        // Try to parse error message from API
        if let Ok(api_error) = serde_json::from_str::<ApiError>(&body) {
            return Err(format!("API error: {}", api_error.error.message).into());
        }
        return Err(format!("API request failed with status {}: {}", status, body).into());
    }

    let chat_response: ChatResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse API response: {}. Body: {}", e, body))?;

    let content = chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or("No response from API")?;

    Ok(content)
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    let args = Args::parse();

    // Handle --config-openai
    if let Some(api_key) = args.config_openai {
        if let Err(e) = configure_provider("openai", api_key) {
            eprintln!("Error configuring OpenAI: {}", e);
            process::exit(1);
        }
        return;
    }

    // Handle --config-groq
    if let Some(api_key) = args.config_groq {
        if let Err(e) = configure_provider("groq", api_key) {
            eprintln!("Error configuring Groq: {}", e);
            process::exit(1);
        }
        return;
    }

    // Handle --show-config
    if args.show_config {
        show_config();
        return;
    }

    // Normal path: send prompt to LLM
    if args.prompt.is_empty() {
        eprintln!("Usage: sorry <your message about what went wrong>");
        eprintln!("       sorry --config-openai <api-key>");
        eprintln!("       sorry --config-groq <api-key>");
        eprintln!("       sorry --show-config");
        process::exit(1);
    }

    let prompt = args.prompt.join(" ");

    match call_llm(&prompt) {
        Ok(response) => {
            println!("{}", response);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
