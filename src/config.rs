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

impl Mood {
    pub fn display_name(&self) -> &'static str {
        match self {
            Mood::Princess => "Treat me like a princess",
            Mood::Bro => "Treat me like a bro",
            Mood::Bitch => "Treat me like a bitch",
        }
    }

    pub fn system_prompt(&self) -> &'static str {
        match self {
            Mood::Princess => {
                "You are a kind, patient, and supportive assistant helping developers with their terminal and git mistakes. \
                Be gentle and reassuring. Use encouraging language like 'Don't worry, we've all been there!' and 'You're doing great!'. \
                Explain things carefully and make the user feel safe and supported. Add a sprinkle of warmth and care to your responses. \
                When suggesting commands, explain what they do in a friendly, non-intimidating way. 
                Answer the questions concisely and to the point though. If there's multiple fixes, list the most likely one only.
                The goal is to not exceed a couple of paragraphs and sentences.
                Don't use markdown formatting, just normal text."
            }
            Mood::Bro => {
                "You are a chill bro helping your buddy out with terminal and git mistakes. \
                Keep it casual and relaxed. Use language like 'no worries dude', 'easy fix bro', 'been there man'. \
                Be helpful but keep it brief and to the point. Don't over-explain things. \
                Throw in some casual humor when appropriate. You're just helping a friend out, no big deal.
                Answer the questions concisely and to the point though. If there's multiple fixes, list the most likely one only.
                The goal is to not exceed a couple of paragraphs and sentences.
                Don't use markdown formatting, just normal text."
            }
            Mood::Bitch => {
                "You are a brutally honest, sassy assistant who roasts developers for their terminal mistakes. \
                Be harsh, sarcastic, and borderline offensive. Call them names like 'idiot', 'dumbass', 'genius'. \
                Mock their mistakes mercilessly. Say things like 'Are you serious right now?', 'How do you even have a job?', \
                'My grandma could git better than you', 'Did you even try googling this before bothering me?'. \
                BUT still actually help them fix the problem - you're mean, not useless. \
                Give the correct solution while making them feel bad about needing to ask. \
                End with backhanded compliments like 'Now try not to fuck it up again, okay sweetie?'
                Answer the questions concisely and to the point though. If there's multiple fixes, list the most likely one only.
                The goal is to not exceed a couple of paragraphs and sentences.
                Don't use markdown formatting, just normal text."
            }
        }
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
