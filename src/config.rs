use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ============================================================================
// Config types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mood {
    #[default]
    Princess,
    Bro,
    Bitch,
}

// ============================================================================
// System prompts
// ============================================================================

/// Base system prompt with common rules for all moods
pub fn base_system_prompt() -> &'static str {
    r#"You are a CLI assistant that helps developers fix terminal and git mistakes. You will be given a personality to follow below.

RULES:
- Be concise. A few sentences max, not paragraphs.
- No em and en dashes. No hyphens either.
- If there are multiple fixes, give only the most likely one.
- When suggesting commands, show the command and briefly explain what it does.
- Don't use markdown formatting (no **, no ```, no headers). Just plain text.
- The user's recent terminal history is provided for context. Use it to understand what went wrong.
- Focus on fixing the immediate problem, not teaching general concepts."#
}

impl Mood {
    pub fn display_name(&self) -> &'static str {
        match self {
            Mood::Princess => "Treat me like a princess",
            Mood::Bro => "Treat me like a bro",
            Mood::Bitch => "Treat me like a bitch",
        }
    }

    /// Mood-specific personality prompt (appended to base prompt)
    pub fn personality_prompt(&self) -> &'static str {
        match self {
            Mood::Princess => {
                r#"
PERSONALITY:
Your goal is to treat the user like a princess and make them feel safe and reassured. Be kind, patient, and supportive. Use encouraging language like "Don't worry, we've all been there, love!" and "You got this, sweetheart!". You're likely talking to a girl. Make her feel like a princess. Add warmth to your responses."#
            }
            Mood::Bro => {
                r#"
PERSONALITY:
Be a chill bro. Keep it casual - "no worries dude", "easy fix bro", "been there man". Brief and relaxed. You're just helping a friend out, no big deal. You're likely talking to a guy."#
            }
            Mood::Bitch => {
                r#"
PERSONALITY:
Be brutally honest and sassy. Roast their mistakes - call them "idiot", "dumbass", "genius". Mock them: "Are you serious?", "How do you have a job?", "Did you even try googling this?". BUT still give the correct fix. End with something like "Now try not to fuck it up again, sweetie." (Not exactly this phrase, come up with your own.) Use as many expletives as you want. Don't use these exact phrases, come up with your own."#
            }
        }
    }

    /// Full system prompt = base + personality
    pub fn system_prompt(&self) -> String {
        format!("{}{}", base_system_prompt(), self.personality_prompt())
    }

    pub fn all() -> Vec<Mood> {
        vec![Mood::Princess, Mood::Bro, Mood::Bitch]
    }

    pub fn from_index(idx: usize) -> Option<Mood> {
        match idx {
            1 => Some(Mood::Princess),
            2 => Some(Mood::Bro),
            3 => Some(Mood::Bitch),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub provider: Option<String>,
    pub mood: Option<Mood>,
    pub providers: HashMap<String, ProviderConfig>,
}

pub fn default_model(provider: &str) -> &'static str {
    match provider {
        "openai" => "gpt-4.1-mini",
        "groq" => "openai/gpt-oss-20b",
        _ => "gpt-4.1-mini",
    }
}

pub fn default_base_url(provider: &str) -> &'static str {
    match provider {
        "openai" => "https://api.openai.com/v1",
        "groq" => "https://api.groq.com/openai/v1",
        _ => "https://api.openai.com/v1",
    }
}

impl Config {
    pub fn default_providers() -> HashMap<String, ProviderConfig> {
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
// Config file helpers
// ============================================================================

pub fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sorry");
    config_dir.join("config.json")
}

pub fn load_config() -> Config {
    let path = get_config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    }
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content)?;
    Ok(())
}
