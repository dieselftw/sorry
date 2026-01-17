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
    
    let mut candidates = if shell.contains("zsh") {
        vec![
            home.join(".zsh_history"),
            home.join(".zhistory"),
            home.join("Library/History/zsh_history"), // macOS zsh history location
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
            home.join("Library/History/zsh_history"), // macOS zsh history location
        ]
    };

    // Also try expanding ~ in HISTFILE if it wasn't found
    if let Ok(histfile) = env::var("HISTFILE") {
        if histfile.starts_with("~/") {
            let expanded = home.join(&histfile[2..]);
            candidates.insert(0, expanded);
        }
    }

    candidates.into_iter().find(|p| p.exists())
}

/// Parse zsh history format
/// Zsh extended history format: ": timestamp:duration;command"
/// Simple format: just the command
/// Multi-line commands can span multiple lines (continuation lines don't start with ": ")
fn parse_zsh_line(line: &str) -> Option<String> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    
    // Extended history format: ": 1234567890:0;actual command"
    // Or: ": 1234567890:duration;command"
    if line.starts_with(": ") {
        if line.contains(";") {
            if let Some(idx) = line.find(';') {
                let cmd = &line[idx + 1..];
                if !cmd.is_empty() {
                    return Some(cmd.to_string());
                }
            }
        }
        // If it starts with ": " but has no semicolon, it might be malformed
        // Skip it
        return None;
    }
    
    // Simple format - just the command (non-extended history)
    // Or continuation line from multi-line command
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

    // For zsh extended history, we need to handle multi-line commands
    // Commands starting with ": " are new entries, others are continuations
    let mut commands = Vec::new();
    
    if is_zsh {
        let mut current_command = String::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            // New command entry (starts with ": ")
            if line.starts_with(": ") {
                // Save previous command if any
                if !current_command.is_empty() {
                    commands.push(current_command.trim().to_string());
                    current_command.clear();
                }
                
                // Parse new command
                if let Some(cmd) = parse_zsh_line(line) {
                    current_command = cmd;
                }
            } else {
                // Continuation line - append to current command
                if !current_command.is_empty() {
                    current_command.push('\n');
                }
                current_command.push_str(line);
            }
        }
        
        // Don't forget the last command
        if !current_command.is_empty() {
            commands.push(current_command.trim().to_string());
        }
    } else {
        // Bash - simpler, one command per line
        commands = content
            .lines()
            .filter_map(|line| parse_bash_line(line))
            .collect();
    }

    // Filter out sorry commands to avoid recursive context
    commands.retain(|cmd| !cmd.trim().starts_with("sorry"));

    // Get last N commands
    let start = commands.len().saturating_sub(count);
    commands[start..].to_vec()
}

/// Parse commands from a newline-separated string (from shell history command)
pub fn parse_commands_from_string(commands_str: &str) -> Vec<String> {
    commands_str
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .filter(|cmd| !cmd.trim().starts_with("sorry"))
        .collect()
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
