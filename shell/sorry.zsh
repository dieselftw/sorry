# Add this to your ~/.zshrc
# Make sure to set SORRY_BIN to the path where 'sorry' binary is installed
# Example: export SORRY_BIN="$HOME/.cargo/bin/sorry"

# Ensure history is shared and appended immediately
setopt INC_APPEND_HISTORY SHARE_HISTORY

sorry() {
  local count=10
  local args=("$@")
  
  # Check if first argument is a number (count override)
  if [[ "$1" =~ ^[0-9]+$ ]]; then
    count="$1"
    args=("${args[@]:1}")  # Remove first arg
  fi

  # `fc -ln -$count` lists the last $count commands, newest last
  local last_cmds
  last_cmds=$(fc -ln -$count)

  # Get the path to sorry binary (default to cargo bin if not set)
  local sorry_bin="${SORRY_BIN:-$HOME/.cargo/bin/sorry}"
  
  # Call the Rust binary with history commands
  "$sorry_bin" \
    --shell zsh \
    --last-commands "$last_cmds" \
    "${args[@]}"
}
