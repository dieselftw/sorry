use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

// ============================================================================
// Config types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
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

// Available models for each provider
fn openai_models() -> Vec<&'static str> {
    vec![
        "gpt-4.1-mini",
        "gpt-4.1-nano",
        "gpt-4.1",
        "gpt-4o",
        "gpt-4o-mini",
        "o1",
        "o1-mini",
        "o3-mini",
    ]
}

fn groq_models() -> Vec<&'static str> {
    vec![
        "llama-3.3-70b-versatile",
        "llama-3.1-8b-instant",
        "llama3-70b-8192",
        "llama3-8b-8192",
        "mixtral-8x7b-32768",
        "gemma2-9b-it",
    ]
}

fn default_model(provider: &str) -> &'static str {
    match provider {
        "openai" => "gpt-4.1-mini",
        "groq" => "llama-3.3-70b-versatile",
        _ => "gpt-4.1-mini",
    }
}

fn default_base_url(provider: &str) -> &'static str {
    match provider {
        "openai" => "https://api.openai.com/v1",
        "groq" => "https://api.groq.com/openai/v1",
        _ => "https://api.openai.com/v1",
    }
}

impl Config {
    fn default_providers() -> HashMap<String, ProviderConfig> {
        let mut providers = HashMap::new();
        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                api_key: String::new(),
                base_url: default_base_url("openai").to_string(),
                model: default_model("openai").to_string(),
            },
        );
        providers.insert(
            "groq".to_string(),
            ProviderConfig {
                api_key: String::new(),
                base_url: default_base_url("groq").to_string(),
                model: default_model("groq").to_string(),
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
    /// Configure OpenAI (interactive setup)
    #[arg(long = "config-openai")]
    config_openai: bool,

    /// Configure Groq (interactive setup)
    #[arg(long = "config-groq")]
    config_groq: bool,

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

// ============================================================================
// Interactive configuration
// ============================================================================

fn read_line() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or_default();
    input.trim().to_string()
}

fn prompt_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    read_line()
}

fn configure_provider_interactive(provider: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = load_config();

    // Ensure we have default provider configs
    if config.providers.is_empty() {
        config.providers = Config::default_providers();
    }

    println!("\n🔧 Configuring {}\n", provider);

    // Step 1: Get API key
    let api_key = prompt_input("Enter API key: ");
    if api_key.is_empty() {
        return Err("API key cannot be empty.".into());
    }

    // Step 2: Select model
    let models: Vec<&str> = match provider {
        "openai" => openai_models(),
        "groq" => groq_models(),
        _ => vec![],
    };
    let default = default_model(provider);

    println!("\nSuggested models:");
    for model in models.iter() {
        println!("  - {}", model);
    }
    println!();

    let model_input = prompt_input(&format!("Enter model name [default: {}]: ", default));

    let model = if model_input.is_empty() {
        default.to_string()
    } else {
        model_input
    };

    // Update config
    let provider_config = config.providers.entry(provider.to_string()).or_insert_with(|| {
        ProviderConfig {
            api_key: String::new(),
            base_url: default_base_url(provider).to_string(),
            model: default.to_string(),
        }
    });

    provider_config.api_key = api_key;
    provider_config.model = model.clone();

    // Set as active provider
    config.provider = Some(provider.to_string());

    save_config(&config)?;
    
    println!("\n✓ Configured {} with model '{}'", provider, model);
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
            println!("Run 'sorry --config-openai' or 'sorry --config-groq' to set up.");
        }
    }
}

// ============================================================================
// LLM API call
// ============================================================================

fn call_llm(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let config = load_config();

    let provider_name = config.provider.ok_or(
        "No provider configured. Run 'sorry --config-openai' or 'sorry --config-groq' first."
    )?;

    let provider = config.providers.get(&provider_name).ok_or(format!(
        "Provider '{}' not found in config.",
        provider_name
    ))?;

    if provider.api_key.is_empty() {
        return Err(format!(
            "API key not set for provider '{}'. Run 'sorry --config-{}' to configure.",
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
    if args.config_openai {
        if let Err(e) = configure_provider_interactive("openai") {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
        return;
    }

    // Handle --config-groq
    if args.config_groq {
        if let Err(e) = configure_provider_interactive("groq") {
            eprintln!("Error: {}", e);
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
        eprintln!("       sorry --config-openai");
        eprintln!("       sorry --config-groq");
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
