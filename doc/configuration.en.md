# KV Storage System Configuration Guide

## Configuration Overview

The KV storage system uses YAML configuration files to manage system behavior. Configuration files are located in the `config/` directory:

- `config/kvdb.default.yaml`: Default configuration template
- `config/kvdb.yaml`: User configuration file (overrides defaults)

## Configuration Reference

### Basic Configuration

```yaml
# Configuration file version
version: 1

# API key for system authentication
api_key: "abcd"

# Data storage directory
data_dir: "storage"

# Compaction threshold - triggers compaction when garbage ratio reaches this value
compact_threshold: 0.2
```

### User Interface Configuration

```yaml
# Show statistics after query execution
show_stats: false

# Auto-complete partial commands
auto_append_part_cmd: false

# Enable multi-line mode
multi_line: true

# Replace newlines with \\n for display
replace_newline: true

# Show number of affected rows
show_affected: false

# Progress bar color setting
progress_color: ""

# Show progress bar
show_progress: false
```

### Data Encoding Configuration

```yaml
encoding:
  # Default encoding format
  # Options: "base64", "hex", "json"
  # Default: "base64"
  default_format: "base64"
  
  # Enable automatic format detection
  # When decoding without specifying format, system auto-detects data format
  # Default: true
  auto_detect: true
  
  # Batch operation size
  # Controls concurrent processing count for bulk encode/decode operations
  # Default: 100
  batch_size: 100
```

## Encoding Format Details

### Base64 Encoding
- **Purpose**: Text representation of binary data
- **Features**: Standard Base64 encoding, outputs readable ASCII characters
- **Use Cases**: Storing images, files, and other binary data

### Hex Encoding
- **Purpose**: Hexadecimal encoding
- **Features**: Each byte represented by two hexadecimal characters
- **Use Cases**: Debugging, data inspection, cryptographic applications

### JSON Encoding
- **Purpose**: JSON string encoding
- **Features**: Applies JSON escaping to strings
- **Use Cases**: Handling text with special characters and newlines

## Configuration Best Practices

### 1. Encoding Format Selection

```yaml
# Recommended: Choose default format based on data type
encoding:
  default_format: "base64"  # General purpose
  # default_format: "hex"   # Debugging scenarios
  # default_format: "json"  # Text processing scenarios
```

### 2. Performance Optimization

```yaml
# High-performance configuration
encoding:
  default_format: "base64"
  auto_detect: false        # Disable auto-detection for better performance
  batch_size: 500          # Increase batch size

# Memory-constrained configuration
encoding:
  default_format: "base64"
  auto_detect: true
  batch_size: 50           # Reduce batch size
```

### 3. Development Environment

```yaml
# Development/debugging configuration
show_stats: true           # Show statistics
show_progress: true        # Show progress bar
show_affected: true        # Show affected rows
multi_line: true          # Enable multi-line mode

encoding:
  default_format: "hex"    # Easier for debugging
  auto_detect: true        # Enable auto-detection
  batch_size: 10          # Small batches for debugging
```

### 4. Production Environment

```yaml
# Production configuration
show_stats: false         # Disable statistics
show_progress: false      # Disable progress bar
show_affected: false      # Disable affected rows display

encoding:
  default_format: "base64" # Standard encoding format
  auto_detect: true        # Maintain compatibility
  batch_size: 200         # Balance performance and memory usage
```

## Configuration Management

### Configuration Priority

1. File specified by `--config` command line argument
2. `config/kvdb.yaml` user configuration file
3. `config/kvdb.default.yaml` default configuration file

### Configuration Validation

The system validates configuration files at startup:

- Checks for required configuration items
- Validates configuration value types and ranges
- Uses default values for invalid configurations with warnings

### Runtime Configuration Updates

Some configuration items support runtime updates without system restart:

```bash
# Update configuration in CLI
kvcli > .show_progress true
kvcli > .show_stats true
```

## Troubleshooting

### Common Configuration Issues

1. **Unsupported Encoding Format**
   ```
   Error: Unsupported encoding format: xyz
   Solution: Check that default_format is base64, hex, or json
   ```

2. **Batch Size Too Large**
   ```
   Error: Out of memory during batch operation
   Solution: Reduce batch_size configuration value
   ```

3. **Invalid Configuration File Format**
   ```
   Error: Invalid YAML format
   Solution: Check YAML syntax and ensure proper indentation
   ```

### Configuration Debugging

Enable debug mode to view configuration loading process:

```bash
./kvcli --debug --config config/kvdb.yaml
```

This displays detailed configuration loading information to help diagnose configuration issues.