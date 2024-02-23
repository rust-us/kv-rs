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
❯ ./kvcli

██  ██  █        █
██ ██   ██      ██
███      ██    ██
██ ██     ██  ██
██  ██     ████  KV Storage CLI

Welcome to kvcli.
Connecting to Client.

kvcli > 
```

### kvcli help
```doc
❯ ./kvcli --help

██  ██  █        █
██ ██   ██      ██
███      ██    ██
██ ██     ██  ██
██  ██     ████  KV Storage CLI

The various kinds of commands that `command` can execute

Usage: kvcli [OPTIONS] [COMMAND]

Commands:
  login  👤  login sys and check user account!
  help   Print this message or the help of the given subcommand(s)

Options:
  -d, --debug                  debug model
      --help                   Print help information
  -q, --quiet <QUIET>          No output printed to stdout [possible values: true, false]
  -l, --log-level <LOG_LEVEL>  [default: info]
  -n, --non-interactive        Force non-interactive mode
      --query=<QUERY>          Query to execute
  -V, --version                Print version
```

### kvcli debug

```doc
> ./kvcli -d
> ./kvcli --debug

██  ██  █        █
██ ██   ██      ██
███      ██    ██
██ ██     ██  ██
██  ██     ████  KV Storage CLI

Args { debug: true, help: false, cmd: None, quiet: None, log_level: "info", non_interactive: false, query: None }
ConfigLoad { version: 1, api_key: "abcd123456", show_progress: "false", show_stats: "false", multi_line: "false", replace_newline: "false" }

Welcome to kvcli.
Connecting to Client.

kvcli > 
```

### Storage

```doc
❯ ./kvcli

██  ██  █        █
██ ██   ██      ██
███      ██    ██
██ ██     ██  ██
██  ██     ████  KV Storage CLI

Welcome to kvcli.
Connecting to Client.

kvcli > set a_key = 1;
```