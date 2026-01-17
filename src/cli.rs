use std::io::{self, Write};

use crate::config::{
    default_base_url, default_model, load_config, save_config, Config, Mood, ProviderConfig,
};

// ============================================================================
// Interactive helpers
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

// ============================================================================
// Configuration commands
// ============================================================================

pub fn configure_provider_interactive(provider: &str) -> Result<(), Box<dyn std::error::Error>> {
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

    // Step 2: Get model name
    let default = default_model(provider);
    let model_input = prompt_input(&format!("Enter model name ({}): ", default));

    let model = if model_input.is_empty() {
        default.to_string()
    } else {
        model_input
    };

    // Update config
    let provider_config = config
        .providers
        .entry(provider.to_string())
        .or_insert_with(|| ProviderConfig {
            api_key: String::new(),
            base_url: default_base_url(provider).to_string(),
            model: default.to_string(),
        });

    provider_config.api_key = api_key;
    provider_config.model = model.clone();

    // Set as active provider
    config.provider = Some(provider.to_string());

    save_config(&config)?;

    println!("\n✓ Configured {} with model '{}'", provider, model);
    Ok(())
}

pub fn configure_behaviour() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = load_config();

    println!("\n🎭 Configure sorry's behaviour\n");
    println!("Choose a mood:\n");

    for (i, mood) in Mood::all().iter().enumerate() {
        let current = if config.mood.unwrap_or_default() == *mood {
            " (current)"
        } else {
            ""
        };
        println!("  {}. {}{}", i + 1, mood.display_name(), current);
    }
    println!();

    let input = prompt_input("Select mood [1-3]: ");

    if let Ok(idx) = input.parse::<usize>() {
        if let Some(mood) = Mood::from_index(idx) {
            config.mood = Some(mood);
            save_config(&config)?;
            println!("\n✓ Mood set to: {}", mood.display_name());
            return Ok(());
        }
    }

    println!("Invalid selection, mood unchanged.");
    Ok(())
}

pub fn show_config() {
    let config = load_config();

    println!();
    
    // Show mood
    let mood = config.mood.unwrap_or_default();
    println!("Mood: {}", mood.display_name());
    println!();

    // Show provider
    match &config.provider {
        Some(provider) => {
            println!("Provider: {}", provider);
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
            println!("Provider: not configured");
            println!("Run 'sorry --config-openai' or 'sorry --config-groq' to set up.");
        }
    }
    println!();
}
