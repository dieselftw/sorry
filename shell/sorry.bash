# Add this to your ~/.bashrc or ~/.bash_profile
# Make sure to set SORRY_BIN to the path where 'sorry' binary is installed
# Example: export SORRY_BIN="$HOME/.cargo/bin/sorry"

# Ensure history is written immediately
export PROMPT_COMMAND='history -a; history -n; '"$PROMPT_COMMAND"

sorry() {
  local count=10
  local args=("$@")
  
  # Check if first argument is a number (count override)
  if [[ "$1" =~ ^[0-9]+$ ]]; then
    count="$1"
    args=("${args[@]:1}")  # Remove first arg
  fi

  # Get the last $count commands *before* this one
  # `history "$((count + 1))"` prints N+1 entries, last is `sorry` itself
  local last_cmds
  last_cmds=$(
    history "$((count + 1))" \
      | head -n "$count" \
      | sed 's/^[ ]*[0-9]\+[ ]*//'
  )

  # Get the path to sorry binary (default to cargo bin if not set)
  local sorry_bin="${SORRY_BIN:-$HOME/.cargo/bin/sorry}"
  
  # Call the Rust binary with history commands
  "$sorry_bin" \
    --shell bash \
    --last-commands "$last_cmds" \
    "${args[@]}"
}
