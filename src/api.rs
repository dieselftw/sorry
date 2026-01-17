use serde::{Deserialize, Serialize};

use crate::config::load_config;
use crate::history::{format_history_context, get_last_commands};

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
// LLM API call
// ============================================================================

pub fn call_llm(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
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

    let mood = config.mood.unwrap_or_default();
    let system_prompt = mood.system_prompt();

    // Get terminal history context
    let commands = get_last_commands(10);
    let history_context = format_history_context(&commands);

    // Build user message with history context
    let user_message = if history_context.is_empty() {
        prompt.to_string()
    } else {
        format!("{}My question/problem: {}", history_context, prompt)
    };

    let url = format!("{}/chat/completions", provider.base_url);

    let request_body = ChatRequest {
        model: provider.model.clone(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_message,
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
