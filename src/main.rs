use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "pipestream")]
#[command(about = "Lightweight ETL pipeline for data transformation", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a pipeline from config file
    Run {
        /// Pipeline configuration file (YAML)
        config: String,
    },
    
    /// Validate pipeline configuration
    Validate {
        /// Pipeline configuration file
        config: String,
    },
    
    /// List available source/destination types
    ListTypes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub name: String,
    pub source: SourceConfig,
    pub transforms: Vec<TransformConfig>,
    pub destinations: Vec<DestinationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    #[serde(rename = "type")]
    pub source_type: String,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub transform_type: String,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationConfig {
    #[serde(rename = "type")]
    pub dest_type: String,
    pub config: HashMap<String, serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Run { config } => {
            run_pipeline(&config).await?;
        }
        
        Commands::Validate { config } => {
            let content = std::fs::read_to_string(&config)?;
            let _: PipelineConfig = serde_yaml::from_str(&content)?;
            println!("✅ Configuration valid!");
        }
        
        Commands::ListTypes => {
            println!("📦 Available Source Types:");
            println!("  - http: Fetch JSON from HTTP API");
            println!("  - file: Read from JSON/YAML file");
            println!("  - stdin: Read from stdin");
            
            println!("\n🔧 Available Transforms:");
            println!("  - filter: Filter records by condition");
            println!("  - map: Transform field values");
            println!("  - merge: Merge multiple fields");
            println!("  - template: Apply template to fields");
            
            println!("\n📤 Available Destinations:");
            println!("  - http: POST JSON to HTTP endpoint");
            println!("  - file: Write to JSON/YAML file");
            println!("  - stdout: Write to stdout");
            println!("  - webhook: Send to webhook");
        }
    }
    
    Ok(())
}

async fn run_pipeline(config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    let config_content = std::fs::read_to_string(config_path)?;
    let config: PipelineConfig = serde_yaml::from_str(&config_content)?;
    
    tracing::info!("Running pipeline: {}", config.name);
    
    // Fetch data from source
    let mut data = fetch_source(&config.source).await?;
    tracing::info!("Fetched {} records from source", data.len());
    
    // Apply transforms
    for transform in &config.transforms {
        data = apply_transform(transform, data).await?;
        tracing::info!("Applied transform: {}", transform.name);
    }
    
    // Send to destinations
    for dest in &config.destinations {
        send_to_destination(dest, &data).await?;
        tracing::info!("Sent to destination: {}", dest.dest_type);
    }
    
    println!("✅ Pipeline '{}' completed successfully!", config.name);
    Ok(())
}

async fn fetch_source(source: &SourceConfig) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    match source.source_type.as_str() {
        "http" => {
            let url = source.config.get("url")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'url' in HTTP source config")?;
            
            let method = source.config.get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("GET");
            
            let client = reqwest::Client::new();
            
            let request = match method {
                "POST" => client.post(url),
                "PUT" => client.put(url),
                _ => client.get(url),
            };
            
            let response = request.send().await?;
            let data: serde_json::Value = response.json().await?;
            
            if let Some(arr) = data.as_array() {
                Ok(arr.clone())
            } else {
                Ok(vec![data])
            }
        }
        "file" => {
            let path = source.config.get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'path' in file source config")?;
            
            let content = std::fs::read_to_string(path)?;
            
            // Try JSON first, then YAML
            if let Ok(data) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                return Ok(data);
            }
            
            if let Ok(data) = serde_yaml::from_str::<Vec<serde_json::Value>>(&content) {
                return Ok(data);
            }
            
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                if data.is_array() {
                    return Ok(data.as_array().unwrap().clone());
                }
                return Ok(vec![data]);
            }
            
            Err("Could not parse file as JSON or YAML".into())
        }
        "stdin" => {
            use std::io::{self, Read};
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            
            if let Ok(data) = serde_json::from_str::<Vec<serde_json::Value>>(&input) {
                return Ok(data);
            }
            
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&input) {
                if data.is_array() {
                    return Ok(data.as_array().unwrap().clone());
                }
                return Ok(vec![data]);
            }
            
            Err("Could not parse stdin as JSON".into())
        }
        _ => Err(format!("Unknown source type: {}", source.source_type).into()),
    }
}

async fn apply_transform(transform: &TransformConfig, data: Vec<serde_json::Value>) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    match transform.transform_type.as_str() {
        "filter" => {
            let field = transform.config.get("field")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'field' in filter config")?;
            
            let operator = transform.config.get("operator")
                .and_then(|v| v.as_str())
                .unwrap_or("equals");
            
            let value = transform.config.get("value")
                .cloned()
                .ok_or("Missing 'value' in filter config")?;
            
            let filtered: Vec<serde_json::Value> = data.into_iter()
                .filter(|record| {
                    if let Some(record_value) = record.get(field) {
                        match operator {
                            "equals" => record_value == &value,
                            "not_equals" => record_value != &value,
                            "contains" => {
                                if let (Some(s1), Some(s2)) = (record_value.as_str(), value.as_str()) {
                                    s1.contains(s2)
                                } else {
                                    false
                                }
                            },
                            "greater_than" => {
                                if let (Some(n1), Some(n2)) = (record_value.as_f64(), value.as_f64()) {
                                    n1 > n2
                                } else {
                                    false
                                }
                            },
                            "less_than" => {
                                if let (Some(n1), Some(n2)) = (record_value.as_f64(), value.as_f64()) {
                                    n1 < n2
                                } else {
                                    false
                                }
                            },
                            _ => false,
                        }
                    } else {
                        false
                    }
                })
                .collect();
            
            Ok(filtered)
        }
        "map" => {
            let mappings = transform.config.get("mappings")
                .and_then(|v| v.as_array())
                .ok_or("Missing 'mappings' in map config")?;
            
            let mapped: Vec<serde_json::Value> = data.into_iter()
                .map(|mut record| {
                    if let Some(obj) = record.as_object_mut() {
                        for mapping in mappings {
                            if let (Some(from), Some(to)) = (
                                mapping.get("from").and_then(|v| v.as_str()),
                                mapping.get("to").and_then(|v| v.as_str()),
                            ) {
                                if let Some(value) = obj.get(from).cloned() {
                                    obj.insert(to.to_string(), value);
                                }
                            }
                        }
                    }
                    record
                })
                .collect();
            
            Ok(mapped)
        }
        "merge" => {
            let fields = transform.config.get("fields")
                .and_then(|v| v.as_array())
                .ok_or("Missing 'fields' in merge config")?;
            
            let into = transform.config.get("into")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'into' in merge config")?;
            
            let separator = transform.config.get("separator")
                .and_then(|v| v.as_str())
                .unwrap_or(" ");
            
            let field_names: Vec<&str> = fields.iter().filter_map(|f| f.as_str()).collect();
            
            let merged: Vec<serde_json::Value> = data.into_iter()
                .map(|mut record| {
                    if let Some(obj) = record.as_object_mut() {
                        let parts: Vec<String> = field_names.iter()
                            .filter_map(|f| obj.get(*f).and_then(|v| v.as_str()).map(|s| s.to_string()))
                            .collect();
                        
                        obj.insert(into.to_string(), serde_json::Value::String(parts.join(separator)));
                    }
                    record
                })
                .collect();
            
            Ok(merged)
        }
        _ => Err(format!("Unknown transform type: {}", transform.transform_type).into()),
    }
}

async fn send_to_destination(dest: &DestinationConfig, data: &Vec<serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
    match dest.dest_type.as_str() {
        "http" => {
            let url = dest.config.get("url")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'url' in HTTP destination config")?;
            
            let method = dest.config.get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("POST");
            
            let client = reqwest::Client::new();
            
            let payload = serde_json::json!({
                "data": data
            });
            
            match method {
                "PUT" => client.put(url).json(&payload).send().await?,
                "PATCH" => client.patch(url).json(&payload).send().await?,
                _ => client.post(url).json(&payload).send().await?,
            };
            
            Ok(())
        }
        "file" => {
            let path = dest.config.get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'path' in file destination config")?;
            
            let format = dest.config.get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("json");
            
            let content = match format {
                "yaml" | "yml" => serde_yaml::to_string(data)?,
                _ => serde_json::to_string_pretty(data)?,
            };
            
            std::fs::write(path, content)?;
            Ok(())
        }
        "stdout" => {
            for record in data {
                println!("{}", serde_json::to_string(record)?);
            }
            Ok(())
        }
        "webhook" => {
            let url = dest.config.get("url")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'url' in webhook destination config")?;
            
            let client = reqwest::Client::new();
            
            let payload = serde_json::json!({
                "payload": data,
                "source": "pipestream"
            });
            
            client.post(url).json(&payload).send().await?;
            
            Ok(())
        }
        _ => Err(format!("Unknown destination type: {}", dest.dest_type).into()),
    }
}
