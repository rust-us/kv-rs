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

### Grammar

| 语法     | CMD                | Desc                                        | DEMO                           |
|--------|--------------------|---------------------------------------------|--------------------------------|
| TIME   | TIME               | 返回当前服务器时间                                   |                                |
| SET    | SET <KEY> <VALUE>  | 设置指定 key 的值。                                | SET ob "redis" <br/> SET key 1 |
| GET    | GET <KEY>          | 获取指定 key 的值                                 | GET ob                         |
| DEL    | DEL <KEY>          |                                             | DEL ob                         |
| DELETE | DELETE <KEY>       |                                             | DELETE ob                      |


```doc
❯ ./kvcli

██  ██  █        █
██ ██   ██      ██
███      ██    ██
██ ██     ██  ██
██  ██     ████  KV Storage CLI

Welcome to kvcli.
Connecting to Client.

kvcli > SET ob redis
OK

> GET ob
"redis"

> DEL ob
(integer) 1
```

### Config

### 在 terminal 终端模式下， Refresh Config
| CMD                | Desc                                        |
|--------------------|---------------------------------------------|
| .show_progress 【true | false】 | Show progress [bar] when executing queries.  Default false   |
| .show_stats 【true | false】 | Show stats after executing queries.  Only works with non-interactive mode.  |
| .show_affected 【true | false】 | Show rows affected |
| .auto_append_part_cmd 【true | false】 | fix part cmd options. default false  |
| .auto_append_part_cmd_symbol 【true | false】 | Division symbol  |
| .multi_line 【true | false】 | Multi line mode, default is true. |
| .replace_newline 【true | false】 | whether replace '\n' with '\\n', default true. |