# KV-CLI 测试用例

本目录包含了 kv-cli 项目的测试用例，主要针对新增的编码功能进行测试。

## 测试文件

### session_tests.rs
测试 Session 结构体中编码引擎的集成和功能：

- **test_session_creation_with_default_encoding**: 测试使用默认编码配置创建会话
- **test_session_creation_with_custom_encoding**: 测试使用自定义编码配置创建会话
- **test_encoding_configuration_updates**: 测试运行时更新编码配置
- **test_encoding_engine_access**: 测试编码引擎的访问接口
- **test_batch_size_validation**: 测试批处理大小的验证
- **test_encoding_configuration_persistence**: 测试编码配置的持久化
- **test_encoding_engine_functionality**: 测试编码引擎的基本功能（编码/解码）
- **test_format_detection**: 测试格式自动检测功能
- **test_encoding_error_handling**: 测试编码错误处理
- **test_cache_functionality**: 测试检测缓存功能
- **test_encoding_engine_default_format**: 测试默认格式编码
- **test_multiple_format_support**: 测试多格式支持

### config_tests.rs
测试配置系统中编码相关的功能：

- **test_encoding_config_default**: 测试编码配置的默认值
- **test_encoding_config_validation**: 测试编码配置的验证
- **test_encoding_config_format_conversion**: 测试格式转换功能
- **test_config_load_default**: 测试配置加载的默认行为
- **test_config_load_encoding_methods**: 测试配置中编码方法
- **test_config_load_batch_size_validation**: 测试批处理大小验证
- **test_config_load_encoding_config_persistence**: 测试编码配置持久化
- **test_config_load_inject_cmd**: 测试命令注入功能
- **test_config_load_validation**: 测试配置验证
- **test_encoding_format_string_parsing**: 测试编码格式字符串解析

## 运行测试

### 运行所有测试
```bash
cargo test -p kvcli
```

### 运行特定测试文件
```bash
# 运行会话测试
cargo test -p kvcli --test session_tests

# 运行配置测试
cargo test -p kvcli --test config_tests
```

### 运行特定测试
```bash
# 运行特定的测试函数
cargo test -p kvcli test_encoding_engine_functionality
```

## 测试覆盖范围

这些测试覆盖了以下功能：

1. **编码引擎集成**: 验证编码引擎正确集成到 Session 中
2. **配置管理**: 测试编码相关配置的加载、验证和更新
3. **格式支持**: 验证 Base64、Hex、JSON 三种格式的支持
4. **自动检测**: 测试格式自动检测功能
5. **错误处理**: 验证各种错误情况的处理
6. **缓存机制**: 测试检测结果的缓存功能
7. **批处理**: 验证批处理大小的配置和验证

## 注意事项

- 测试使用临时目录避免文件锁定冲突
- 异步测试使用 `#[tokio::test]` 宏
- 配置测试不依赖外部文件系统
- 所有测试都是独立的，可以并行运行
