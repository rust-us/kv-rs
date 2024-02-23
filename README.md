# kv
KV Storage in Rust

## Components
[kv](./kv): KV Storage

[kv-cli](./kv-cli): KV CLI


## Installation for
### Cargo
> cargo install kvcli

## Usage

### kvcli
```doc
❯ kvcli


██  ██  █        █
██ ██   ██      ██
███      ██    ██
██ ██    ██  ██
██  ██     ████

Config ConfigLoad { version: 0, api_key: "", show_progress: "false", show_stats: "false", multi_line: "false", replace_newline: "false" }
ArgsArgs { help: false, cmd: None, quiet: None, log_level: "info", non_interactive: false, query: None }

Welcome to kvcli.
Connecting to Client.

kvcli > 
```

### kvcli help
```doc
❯ kvcli --help

██  ██  █        █
██ ██   ██      ██
███      ██    ██
██ ██    ██  ██
██  ██     ████

Config ConfigLoad { version: 0, api_key: "", show_progress: "false", show_stats: "false", multi_line: "false", replace_newline: "false" }
ArgsArgs { help: true, cmd: None, quiet: None, log_level: "info", non_interactive: false, query: None }

The various kinds of commands that `command` can execute

Usage: kvcli [OPTIONS] [COMMAND]

Commands:
  login  👤  login sys and check user account!
  help   Print this message or the help of the given subcommand(s)

Options:
      --help                   Print help information
  -q, --quiet <QUIET>          No output printed to stdout [possible values: true, false]
  -l, --log-level <LOG_LEVEL>  [default: info]
  -n, --non-interactive        Force non-interactive mode
      --query=<QUERY>          Query to execute
  -V, --version                Print version
```