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

A distributed kv storage CLI

Usage: kvcli [OPTIONS] [COMMAND]

Commands:
  login  👤  login sys and check user account!
  help   Print this message or the help of the given subcommand(s)

Options:
  -d, --debug                       debug model
      --help                        Print help information
      --storage_path <CONFIG_PATH>  load config path, default '${pwd}/config' [default: config]
  -q, --quiet <QUIET>               No output printed to stdout [possible values: true, false]
  -l, --log-level <LOG_LEVEL>       [default: info]
  -n, --non-interactive             Force non-interactive mode
      --query=<QUERY>               Query to execute
  -V, --version                     Print version
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
| INFO   | INFO               | 查看服务状态和相关信息                                 | INFO                           |
| TIME   | TIME               | 返回当前服务器时间                                   | TIME                           |
| KSIZE  | KSIZE              | 返回当前库文件的 key 的数量                            | KSIZE                          |
| EXIT   | exit               | 关闭当前连接                                      | exit                           |
| SHOW   | SHOW DB            | 显示当前使用的库文件                                  | SHOW DB                        |
| SELECT | SELECT <db>        | 切换到指定的库文件                                   |                                |
| SET    | SET <KEY> <VALUE>  | 设置指定 key 的值。                                | SET ob "redis" <br/> SET key 1 |
| GET    | GET <KEY>          | 获取指定 key 的值                                 | GET ob                         |
| KEYS   | KEYS               | key list                                    | KEYS                           |
| DEL    | DEL <KEY>          |                                             | DEL ob                         |
| DELETE | DELETE <KEY>       |                                             | DELETE ob                      |
| GETSET | GETSET key value   | 将给定 key 的值设为 value ，并返回 key 的旧值(old value)。 |                                |
| MGET   | MGET key1 [key2..] | 获取所有(一个或多个)给定 key 的值。                       |                                |
| SETEX  | SETNX key value    | 只有在 key 不存在时设置 key 的值。                      |                                |


```doc
❯ ./kvcli

██  ██  █        █
██ ██   ██      ██
███      ██    ██
██ ██     ██  ██
██  ██     ████  KV Storage CLI

Welcome to kvcli.
Connecting to Client.

kvcli > SET order_key xxx
OK ~

kvcli > keys
order_key

kvcli > ksize
1

kvcli > GET order_key
xxx

kvcli > DEL order_key
OK ~

kvcli > GET order_key
N/A ~

```

### Config

### 在 terminal 终端模式下， Refresh Config
| CMD                               | Desc                                        |
|-----------------------------------|---------------------------------------------|
| .show_progress 【true, false】      | Show progress [bar] when executing queries.  Default false   |
| .show_stats 【true, false】 | Show stats after executing queries.  Only works with non-interactive mode.  |
| .show_affected 【true, false】 | Show rows affected |
| .auto_append_part_cmd 【true, false】 | fix part cmd options. default false  |
| .auto_append_part_cmd_symbol 【true, false】 | Division symbol  |
| .multi_line 【true, false】 | Multi line mode, default is true. |
| .replace_newline 【true, false】 | whether replace '\n' with '\\n', default true. |