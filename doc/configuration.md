# KV存储系统配置文档

## 配置文件概述

KV存储系统使用YAML格式的配置文件来管理系统行为。配置文件位于 `config/` 目录下：

- `config/kvdb.default.yaml`: 默认配置模板
- `config/kvdb.yaml`: 用户配置文件（覆盖默认设置）

## 配置项详解

### 基础配置

```yaml
# 配置文件版本
version: 1

# API密钥，用于系统认证
api_key: "abcd"

# 数据存储目录
data_dir: "storage"

# 压缩阈值，当垃圾数据比例达到此值时触发压缩
compact_threshold: 0.2
```

### 用户界面配置

```yaml
# 显示查询执行后的统计信息
show_stats: false

# 自动补全部分命令
auto_append_part_cmd: false

# 启用多行模式
multi_line: true

# 将换行符替换为 \\n 显示
replace_newline: true

# 显示受影响的行数
show_affected: false

# 进度条颜色设置
progress_color: ""

# 显示进度条
show_progress: false
```

### 数据编码配置

```yaml
encoding:
  # 默认编码格式
  # 可选值: "base64", "hex", "json"
  # 默认: "base64"
  default_format: "base64"
  
  # 启用自动格式检测
  # 当解码时未指定格式，系统会自动检测数据格式
  # 默认: true
  auto_detect: true
  
  # 批量操作的批次大小
  # 控制批量编码/解码操作的并发处理数量
  # 默认: 100
  batch_size: 100
```

## 编码格式说明

### Base64编码
- **用途**: 适用于二进制数据的文本表示
- **特点**: 标准的Base64编码，输出可读的ASCII字符
- **适用场景**: 存储图片、文件等二进制数据

### Hex编码
- **用途**: 十六进制编码
- **特点**: 每个字节用两个十六进制字符表示
- **适用场景**: 调试、数据检查、密码学应用

### JSON编码
- **用途**: JSON字符串编码
- **特点**: 对字符串进行JSON转义处理
- **适用场景**: 处理包含特殊字符、换行符的文本数据

## 配置最佳实践

### 1. 编码格式选择

```yaml
# 推荐配置：根据数据类型选择默认格式
encoding:
  default_format: "base64"  # 通用场景
  # default_format: "hex"   # 调试场景
  # default_format: "json"  # 文本处理场景
```

### 2. 性能优化配置

```yaml
# 高性能场景配置
encoding:
  default_format: "base64"
  auto_detect: false        # 关闭自动检测以提高性能
  batch_size: 500          # 增大批次大小

# 内存受限场景配置
encoding:
  default_format: "base64"
  auto_detect: true
  batch_size: 50           # 减小批次大小
```

### 3. 开发环境配置

```yaml
# 开发调试配置
show_stats: true           # 显示统计信息
show_progress: true        # 显示进度条
show_affected: true        # 显示受影响行数
multi_line: true          # 启用多行模式

encoding:
  default_format: "hex"    # 便于调试查看
  auto_detect: true        # 启用自动检测
  batch_size: 10          # 小批次便于调试
```

### 4. 生产环境配置

```yaml
# 生产环境配置
show_stats: false         # 关闭统计信息
show_progress: false      # 关闭进度条
show_affected: false      # 关闭受影响行数显示

encoding:
  default_format: "base64" # 标准编码格式
  auto_detect: true        # 保持兼容性
  batch_size: 200         # 平衡性能和内存使用
```

## 配置文件管理

### 配置文件优先级

1. 命令行参数 `--config` 指定的文件
2. `config/kvdb.yaml` 用户配置文件
3. `config/kvdb.default.yaml` 默认配置文件

### 配置验证

系统启动时会验证配置文件的有效性：

- 检查必需的配置项是否存在
- 验证配置值的类型和范围
- 对无效配置使用默认值并发出警告

### 运行时配置更新

部分配置项支持运行时更新，无需重启系统：

```bash
# 在CLI中更新配置
kvcli > .show_progress true
kvcli > .show_stats true
```

## 故障排除

### 常见配置问题

1. **编码格式不支持**
   ```
   错误: Unsupported encoding format: xyz
   解决: 检查 default_format 是否为 base64、hex 或 json
   ```

2. **批次大小过大**
   ```
   错误: Out of memory during batch operation
   解决: 减小 batch_size 配置值
   ```

3. **配置文件格式错误**
   ```
   错误: Invalid YAML format
   解决: 检查YAML语法，确保缩进正确
   ```

### 配置调试

启用调试模式查看配置加载过程：

```bash
./kvcli --debug --config config/kvdb.yaml
```

这将显示详细的配置加载信息，帮助诊断配置问题。