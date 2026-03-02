# PipeStream

Lightweight ETL pipeline tool for data transformation and routing. Connect APIs, transform data with templates, and route to multiple destinations.

## Features

- 🌊 **Pipeline Config** - Define pipelines in YAML
- 📥 **Multiple Sources** - HTTP APIs, files, stdin
- 🔧 **Transforms** - Filter, map, merge, template
- 📤 **Multiple Destinations** - HTTP endpoints, files, webhooks

## Quick Start

```bash
# Install
cargo install pipestream

# Create pipeline config
cat > pipeline.yaml << 'EOF'
name: my-pipeline
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
EOF

# Run pipeline
pipestream run pipeline.yaml

# List available types
pipestream list-types
```

## Configuration

See `examples/` for more configuration examples.

## License

MIT
