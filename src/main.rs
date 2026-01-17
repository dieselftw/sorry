mod api;
mod cli;
mod config;
mod history;

use clap::Parser;
use std::process;

use api::call_llm;
use cli::{configure_behaviour, configure_provider_interactive, show_config};

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

    /// Configure sorry's behaviour/mood
    #[arg(long = "behaviour")]
    behaviour: bool,

    /// Show current configuration (without revealing keys)
    #[arg(long = "show-config")]
    show_config: bool,

    /// Shell type (bash/zsh) - used when --last-commands is provided
    #[arg(long = "shell")]
    shell: Option<String>,

    /// Last commands from shell history (newline-separated)
    #[arg(long = "last-commands")]
    last_commands: Option<String>,

    /// The prompt to send to the LLM
    #[arg(trailing_var_arg = true)]
    prompt: Vec<String>,
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

    // Handle --behaviour
    if args.behaviour {
        if let Err(e) = configure_behaviour() {
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
        eprintln!("       sorry --behaviour");
        eprintln!("       sorry --show-config");
        process::exit(1);
    }

    let prompt = args.prompt.join(" ");

    match call_llm(&prompt, args.last_commands.as_deref()) {
        Ok(response) => {
            println!("{}", response);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
