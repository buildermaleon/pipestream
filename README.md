# PipeStream 🌊

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-1.85+-dea584?style=flat&logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Pipeline](https://img.shields.io/badge/Pipeline-ETL-blue.svg)]()

**Lightweight ETL pipeline for data transformation and routing.**

</div>

## Overview

PipeStream is a blazing-fast ETL (Extract, Transform, Load) pipeline tool built in Rust. It connects to various data sources, applies transformations, and routes data to multiple destinations - all defined in simple YAML configuration files.

## Features

| Feature | Description |
|---------|-------------|
| 📥 **Multiple Sources** | HTTP APIs, files (JSON/YAML), stdin |
| 🔧 **Powerful Transforms** | Filter, map, merge, template transformations |
| 📤 **Flexible Outputs** | HTTP endpoints, files, stdout, webhooks |
| ⚡ **High Performance** | Built in Rust for maximum speed |
| 📝 **YAML Config** | Simple, declarative pipeline definitions |
| 🔄 **Streaming** | Process data in real-time |

## Quick Start

### Installation

```bash
cargo install pipestream
```

### Create a Pipeline

```yaml
# pipeline.yaml
name: my-data-pipeline
source:
  type: http
  config:
    url: https://api.example.com/data

transforms:
  - name: filter-active
    type: filter
    config:
      field: status
      operator: equals
      value: active

destinations:
  - type: stdout
```

### Run

```bash
pipestream run pipeline.yaml

# List available source/destination types
pipestream list-types
```

## Configuration Reference

### Sources

| Type | Config | Description |
|------|---------|-------------|
| `http` | `url`, `method` | Fetch from HTTP API |
| `file` | `path` | Read from JSON/YAML file |
| `stdin` | - | Read from standard input |

### Transforms

| Type | Config | Description |
|------|--------|-------------|
| `filter` | `field`, `operator`, `value` | Filter records by condition |
| `map` | `mappings` | Rename/map fields |
| `merge` | `fields`, `into`, `separator` | Combine fields |
| `template` | `template`, `output` | Apply template to fields |

### Destinations

| Type | Config | Description |
|------|---------|-------------|
| `http` | `url`, `method` | POST/PUT to endpoint |
| `file` | `path`, `format` | Write to file |
| `stdout` | - | Print to console |
| `webhook` | `url` | Send to webhook |

## Use Cases

- 🔄 **Data Synchronization** - Move data between systems
- 🔄 **API Aggregation** - Combine multiple API responses
- 🔄 **Data Cleaning** - Filter and transform raw data
- 🔄 **Webhook Routing** - Forward events to multiple endpoints
- 🔄 **ETL Pipelines** - Simple ETL without heavy infrastructure

## Performance

- ⚡ Blazing fast - built in Rust
- 📦 ~5MB binary
- 💨 Processes thousands of records per second

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<div align="center">

Built with 🔥 by [Anvil](https://github.com/buildermaleon)

</div>
