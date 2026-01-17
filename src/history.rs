use std::env;
use std::fs;
use std::path::PathBuf;

/// Get the path to the shell history file
fn get_history_path() -> Option<PathBuf> {
    // Check HISTFILE env var first (works for most shells)
    if let Ok(histfile) = env::var("HISTFILE") {
        let path = PathBuf::from(histfile);
        if path.exists() {
            return Some(path);
        }
    }

    // Fall back to common locations
    let home = dirs::home_dir()?;
    
    // Check which shell is being used
    let shell = env::var("SHELL").unwrap_or_default();
    
    let candidates = if shell.contains("zsh") {
        vec![
            home.join(".zsh_history"),
            home.join(".zhistory"),
        ]
    } else if shell.contains("bash") {
        vec![
            home.join(".bash_history"),
        ]
    } else {
        // Try common ones
        vec![
            home.join(".zsh_history"),
            home.join(".bash_history"),
            home.join(".zhistory"),
        ]
    };

    candidates.into_iter().find(|p| p.exists())
}

/// Parse zsh history format
/// Zsh extended history format: ": timestamp:0;command"
/// Simple format: just the command
fn parse_zsh_line(line: &str) -> Option<String> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    
    // Extended history format: ": 1234567890:0;actual command"
    if line.starts_with(": ") && line.contains(";") {
        if let Some(idx) = line.find(';') {
            let cmd = &line[idx + 1..];
            if !cmd.is_empty() {
                return Some(cmd.to_string());
            }
        }
        return None;
    }
    
    // Simple format - just the command
    Some(line.to_string())
}

/// Parse bash history format (simpler - just commands)
fn parse_bash_line(line: &str) -> Option<String> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    Some(line.to_string())
}

/// Get the last N commands from shell history
pub fn get_last_commands(count: usize) -> Vec<String> {
    let Some(history_path) = get_history_path() else {
        return Vec::new();
    };

    let Ok(content) = fs::read_to_string(&history_path) else {
        return Vec::new();
    };

    let is_zsh = history_path
        .to_string_lossy()
        .contains("zsh");

    let commands: Vec<String> = content
        .lines()
        .filter_map(|line| {
            if is_zsh {
                parse_zsh_line(line)
            } else {
                parse_bash_line(line)
            }
        })
        // Filter out sorry commands to avoid recursive context
        .filter(|cmd| !cmd.starts_with("sorry"))
        .collect();

    // Get last N commands
    let start = commands.len().saturating_sub(count);
    commands[start..].to_vec()
}

/// Format commands for inclusion in prompt
pub fn format_history_context(commands: &[String]) -> String {
    if commands.is_empty() {
        return String::new();
    }

    let mut context = String::from("Here are my last terminal commands:\n```\n");
    for (i, cmd) in commands.iter().enumerate() {
        context.push_str(&format!("{}. {}\n", i + 1, cmd));
    }
    context.push_str("```\n\n");
    context
}
