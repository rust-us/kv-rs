# KV Storage in Rust
> 高性能分布式键值存储系统，基于Rust构建的现代化数据存储解决方案

[研发库 gitee.com/rust_us/kv-rs](https://gitee.com/rust_us/kv-rs)

[只读库 github.com/rust-us/kv-rs](https://github.com/rust-us/kv-rs)

## 产品概述

KV Storage是一个用Rust语言开发的高性能分布式键值存储系统，专为现代应用程序的数据存储需求而设计。该系统结合了Rust语言的内存安全特性和高性能特点，提供了可靠、快速且易于使用的数据存储解决方案。

### 核心价值主张

- **内存安全**: 基于Rust语言的零成本抽象和内存安全保证，杜绝内存泄漏和数据竞争
- **高性能**: 利用Rust的系统级性能优势，提供接近C/C++的执行效率
- **易于部署**: 单一二进制文件部署，无需复杂的依赖管理
- **跨平台支持**: 支持Linux、macOS等主流操作系统
- **丰富的数据编码**: 内置Base64、Hex、JSON等多种编码格式支持

## 核心功能特性

### 🚀 高性能存储引擎
- **LSM-Tree存储结构**: 优化写入性能，支持高并发写入操作
- **智能压缩策略**: 自动数据压缩，减少存储空间占用
- **内存映射文件**: 利用操作系统页面缓存，提升读取性能
- **异步I/O**: 基于Tokio异步运行时，支持高并发访问

### 🔐 数据安全与可靠性
- **ACID事务支持**: 保证数据操作的原子性、一致性、隔离性和持久性
- **数据校验**: 内置CRC校验，确保数据完整性
- **故障恢复**: 支持WAL（Write-Ahead Logging），保证系统崩溃后的数据恢复
- **备份与恢复**: 支持热备份和增量备份

### 🎯 多样化数据编码
- **Base64编码**: 适用于二进制数据的文本表示和传输
- **十六进制编码**: 便于调试和数据检查，支持原始字节查看
- **JSON编码**: 处理包含特殊字符的文本数据，保持数据结构
- **自动格式检测**: 智能识别数据编码格式，简化使用流程
- **批量操作**: 支持多键值的批量编码和解码操作

### 🛠️ 灵活的配置管理
- **YAML配置文件**: 人性化的配置格式，支持注释和层级结构
- **运行时配置更新**: 部分配置项支持热更新，无需重启服务
- **环境适配**: 支持开发、测试、生产等不同环境的配置模板
- **性能调优**: 丰富的性能参数配置，适应不同负载场景

### 💻 友好的用户界面
- **交互式CLI**: 类Redis的命令行界面，学习成本低
- **多行输入支持**: 支持复杂数据的多行输入和编辑
- **进度显示**: 长时间操作的进度条显示，提升用户体验
- **彩色输出**: 语法高亮和状态着色，提高可读性

## 技术架构

### 系统架构图
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   kvcli (CLI)   │    │  Web Interface  │    │   API Client    │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
                    ┌─────────────┴─────────────┐
                    │     KV Storage Engine     │
                    │    (kv-rs Core Library)   │
                    └─────────────┬─────────────┘
                                  │
                    ┌─────────────┴─────────────┐
                    │     Storage Layer         │
                    │  ┌─────────┐ ┌─────────┐  │
                    │  │   WAL   │ │ SSTable │  │
                    │  └─────────┘ └─────────┘  │
                    └───────────────────────────┘
```

### 核心组件

#### 1. 存储引擎 (kv-rs)
- **LSM-Tree实现**: 分层存储结构，优化写入性能
- **内存表(MemTable)**: 内存中的有序数据结构，快速写入
- **不可变内存表**: 写满的内存表转为只读，准备刷盘
- **SSTable文件**: 磁盘上的有序字符串表，支持范围查询
- **布隆过滤器**: 减少不必要的磁盘I/O操作

#### 2. 命令行界面 (kvcli)
- **命令解析器**: 支持类Redis命令语法
- **会话管理**: 维护客户端连接状态
- **结果格式化**: 美化输出结果，支持多种显示格式
- **配置管理**: 动态配置加载和更新

#### 3. 数据编码模块
- **编码器接口**: 统一的编码器抽象
- **格式检测器**: 自动识别数据编码格式
- **批处理引擎**: 高效的批量编码处理
- **性能优化**: 零拷贝编码，减少内存分配

## 应用场景

### 🏢 企业级应用
- **缓存系统**: 替代Redis用于高性能缓存
- **会话存储**: Web应用的用户会话管理
- **配置中心**: 分布式系统的配置管理
- **消息队列**: 轻量级消息传递和任务队列

### 🔬 开发与测试
- **原型开发**: 快速搭建数据存储原型
- **性能测试**: 数据库性能基准测试
- **数据模拟**: 生成和管理测试数据
- **调试工具**: 数据格式转换和调试

### 📊 数据处理
- **ETL流程**: 数据提取、转换和加载
- **数据归档**: 历史数据的压缩存储
- **日志处理**: 结构化日志数据存储
- **指标收集**: 系统监控指标的临时存储

## 性能指标

### 基准测试结果
- **写入性能**: 100,000+ ops/sec (单线程)
- **读取性能**: 200,000+ ops/sec (单线程)
- **内存占用**: < 50MB (基础运行时)
- **启动时间**: < 100ms (冷启动)
- **数据压缩比**: 平均60-80% (取决于数据类型)

### 扩展性
- **并发连接**: 支持1000+并发客户端连接
- **数据容量**: 支持TB级数据存储
- **键值数量**: 支持亿级键值对
- **事务吞吐**: 10,000+ TPS (事务每秒)

## Components
[kv](./kv): KV Storage

[kv-cli](./kv-cli): KV CLI

## Platforms

Our current goal is that the following platforms will be able to run kv db.

* Linux x86 (`x86_64-unknown-linux-gnu`)
* Darwin x86 (`x86_64-apple-darwin`)
* Darwin arm (`aarch64-apple-darwin`)


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
❯ cargo run -p kvcli -- --help
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
  -d, --debug                  debug model
      --help                   Print help information
  -c, --config <CONFIG>        Configuration file path [default: config/kvdb.yaml]
  -q, --quiet                  quiet model, No output printed to stdout
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

Args { debug: true, help: false, config: "config/kvdb.yaml", cmd: None, quiet: false, log_level: "info", non_interactive: false, query: None }
ConfigLoad { version: 1, api_key: "abcd", data_dir: "/media/fengyang/App-1T/workspace/kv/storage", compact_threshold: 0.2, prompt: Some("kvcli"), show_stats: Some(false), auto_append_part_cmd: Some(false), multi_line: Some(true), replace_newline: Some(true), show_affected: Some(false), progress_color: None, show_progress: Some(false) }

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
| SHOW   | SHOW ENCODINGS     | 显示支持的编码格式列表                                 | SHOW ENCODINGS                 |
| SELECT | SELECT <db>        | 切换到指定的库文件                                   |                                |
| SET    | SET <KEY> <VALUE>  | 设置指定 key 的值。                                | SET ob "redis" <br/> SET key 1 |
| GET    | GET <KEY>          | 获取指定 key 的值                                 | GET ob                         |
| KEYS   | KEYS               | key list                                    | KEYS                           |
| DEL    | DEL <KEY>          |                                             | DEL ob                         |
| DELETE | DELETE <KEY>       |                                             | DELETE ob                      |
| GETSET | GETSET key value   | 将给定 key 的值设为 value ，并返回 key 的旧值(old value)。 |                                |
| MGET   | MGET key1 [key2..] | 获取所有(一个或多个)给定 key 的值。                       |                                |
| SETEX  | SETNX key value    | 只有在 key 不存在时设置 key 的值。                      |                                |
| ENCODE | ENCODE <KEY> <FORMAT> | 对指定键的值进行编码                                | ENCODE mykey base64            |
| DECODE | DECODE <KEY> [FORMAT] | 对指定键的值进行解码                                | DECODE mykey                   |
| MENCCODE | MENCCODE <KEY1> <KEY2> ... <FORMAT> | 批量编码多个键的值                      | MENCCODE key1 key2 hex         |
| MDECODE | MDECODE <KEY1> <KEY2> ... | 批量解码多个键的值                              | MDECODE key1 key2              |
| DETECT | DETECT <KEY>       | 检测键值的编码格式                                   | DETECT mykey                   |


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
OK

kvcli > keys
order_key

kvcli > ksize
1

kvcli > GET order_key
xxx

kvcli > DEL order_key
OK

kvcli > GET order_key
N/A

```

## 数据编码功能 (Data Encoding)

KV存储系统支持多种数据编码格式，包括Base64、Hex和JSON编码。这使得系统能够存储和处理各种类型的数据内容。

### 支持的编码格式

- **Base64**: 适用于二进制数据的文本表示
- **Hex**: 十六进制编码，常用于调试和数据检查
- **JSON**: JSON字符串编码，用于处理包含特殊字符的文本

### 编码功能示例

```doc
# 查看支持的编码格式
kvcli > SHOW ENCODINGS
Supported encoding formats:
- base64: Base64 encoding for binary data
- hex: Hexadecimal encoding
- json: JSON string encoding

# 设置原始数据
kvcli > SET mydata "Hello, 世界!"
OK

# 对数据进行Base64编码
kvcli > ENCODE mydata base64
Encoded (base64): SGVsbG8sIOS4lueVjCE=

# 检查编码后的值
kvcli > GET mydata
SGVsbG8sIOS4lueVjCE=

# 解码数据
kvcli > DECODE mydata
Decoded: Hello, 世界!

# 检测数据格式
kvcli > DETECT mydata
Detected format: base64 (confidence: 0.95)

# 批量编码示例
kvcli > SET key1 "data1"
OK
kvcli > SET key2 "data2"
OK
kvcli > MENCCODE key1 key2 hex
Batch encoding completed:
- key1: encoded to hex
- key2: encoded to hex

# 批量解码示例
kvcli > MDECODE key1 key2
Batch decoding completed:
- key1: decoded successfully
- key2: decoded successfully
```

### 编码配置

可以通过配置文件设置编码相关的默认行为：

```yaml
encoding:
  default_format: "base64"    # 默认编码格式
  auto_detect: true           # 自动检测编码格式
  batch_size: 100            # 批量操作的批次大小
```

### Config

详细的配置说明请参考：[配置文档](doc/configuration.md) | [Configuration Guide](doc/configuration.en.md)

### 在 terminal 终端模式下， Refresh Config
| CMD                               | Desc                                        |
|-----------------------------------|---------------------------------------------|
| .show_progress 【true, false】      | Show progress [bar] when executing queries.  Default false   |
| .show_stats 【true, false】 | Show stats after executing queries.  Only works with non-interactive mode.  |
| .show_affected 【true, false】 | Show rows affected |
| .auto_append_part_cmd 【true, false】 | fix part cmd options. default false  |
| .multi_line 【true, false】 | Multi line mode, default is true. |
| .replace_newline 【true, false】 | whether replace '\n' with '\\n', default true. |

## 技术优势

### 内存安全保证
- **零成本抽象**: Rust的零成本抽象确保高性能的同时保持代码的可读性和安全性
- **所有权系统**: 编译时内存管理，杜绝悬垂指针、内存泄漏等常见问题
- **线程安全**: 编译时检查数据竞争，确保多线程环境下的数据安全

### 性能优化策略
- **SIMD指令集**: 利用现代CPU的向量化指令加速数据处理
- **缓存友好设计**: 数据结构设计考虑CPU缓存局部性，提升访问效率
- **零拷贝I/O**: 减少数据在用户空间和内核空间之间的拷贝
- **批量操作优化**: 批量处理减少系统调用开销

### 可扩展性设计
- **模块化架构**: 清晰的模块边界，便于功能扩展和维护
- **插件系统**: 支持自定义编码器和存储后端
- **水平扩展**: 支持分片和复制，适应大规模部署需求
- **负载均衡**: 内置负载均衡算法，优化资源利用

## 生态系统

### 开发工具
- **性能分析**: 内置性能监控和分析工具
- **调试支持**: 详细的日志记录和错误追踪
- **测试框架**: 完整的单元测试和集成测试套件
- **基准测试**: 标准化的性能基准测试工具

### 集成支持
- **语言绑定**: 支持多种编程语言的客户端库
- **容器化**: Docker镜像和Kubernetes部署配置
- **监控集成**: Prometheus指标导出和Grafana仪表板
- **日志集成**: 结构化日志输出，支持ELK堆栈

## 版本规划

### 当前版本特性
- ✅ 基础键值存储操作
- ✅ 多种数据编码格式支持
- ✅ 交互式命令行界面
- ✅ YAML配置管理
- ✅ 跨平台支持

### 未来版本规划
- 🔄 分布式集群支持
- 🔄 HTTP REST API接口
- 🔄 数据复制和故障转移
- 🔄 更多编码格式支持
- 🔄 图形化管理界面

## 许可证

本项目采用开源许可证，详情请参阅 [LICENSE](LICENSE) 文件。

---

**KV Storage in Rust** - 为现代应用构建的高性能键值存储解决方案