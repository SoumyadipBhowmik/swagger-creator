use std::fs::{self, File, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use clap::Parser;
use postman_to_swagger::{convert_postman_to_openapi, parse_postman_collection};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input Postman collection filename
    #[arg(long)]
    input: Option<String>,

    /// Output OpenAPI YAML filename
    #[arg(long)]
    output: Option<String>,

    /// Input directory for Postman collections
    #[arg(long, default_value = "collections")]
    input_dir: String,

    /// Output directory for OpenAPI specs
    #[arg(long, default_value = "output")]
    output_dir: String,
}

fn main() {
    let cli = Cli::parse();
    
    // Create output directory if it doesn't exist
    if !Path::new(&cli.output_dir).exists() {
        if let Err(e) = create_dir_all(&cli.output_dir) {
            eprintln!("Failed to create output directory: {}", e);
            process::exit(1);
        }
        println!("Created output directory: {}", cli.output_dir);
    }
    
    // Process a single file if specified
    if let Some(input_file) = &cli.input {
        let input_path = PathBuf::from(&cli.input_dir).join(input_file);
        
        // Determine output filename
        let output_filename = match &cli.output {
            Some(output) => output.to_string(),
            None => {
                let base_name = Path::new(input_file)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                format!("{}_openapi.yaml", base_name)
            }
        };
        
        let output_path = PathBuf::from(&cli.output_dir).join(&output_filename);
        
        // Convert the collection
        println!("Processing: {}", input_path.display());
        match parse_postman_collection(&input_path) {
            Ok(postman_collection) => {
                let openapi_spec = convert_postman_to_openapi(&postman_collection);
                
                match File::create(&output_path) {
                    Ok(mut file) => {
                        match serde_yaml::to_string(&openapi_spec) {
                            Ok(yaml) => {
                                if let Err(e) = file.write_all(yaml.as_bytes()) {
                                    eprintln!("Failed to write to file: {}", e);
                                    process::exit(1);
                                }
                                println!("Converted to {}", output_path.display());
                            },
                            Err(e) => {
                                eprintln!("Failed to serialize to YAML: {}", e);
                                process::exit(1);
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to create output file: {}", e);
                        process::exit(1);
                    }
                }
            },
            Err(e) => {
                eprintln!("Error reading Postman collection: {}", e);
                process::exit(1);
            }
        }
    } 
    // Process all files in input directory
    else {
        let input_path = Path::new(&cli.input_dir);
        if !input_path.exists() {
            eprintln!("Input directory '{}' does not exist. Please create it and add your Postman collections.", cli.input_dir);
            process::exit(1);
        }
        
        let mut files_processed = 0;
        
        match fs::read_dir(input_path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        let base_name = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("output");
                        
                        let output_path = PathBuf::from(&cli.output_dir)
                            .join(format!("{}_openapi.yaml", base_name));
                        
                        println!("Processing: {}", path.display());
                        match parse_postman_collection(&path) {
                            Ok(postman_collection) => {
                                let openapi_spec = convert_postman_to_openapi(&postman_collection);
                                
                                match File::create(&output_path) {
                                    Ok(mut file) => {
                                        match serde_yaml::to_string(&openapi_spec) {
                                            Ok(yaml) => {
                                                if let Err(e) = file.write_all(yaml.as_bytes()) {
                                                    eprintln!("Failed to write to file: {}", e);
                                                    continue;
                                                }
                                                println!("Converted to {}", output_path.display());
                                                files_processed += 1;
                                            },
                                            Err(e) => {
                                                eprintln!("Failed to serialize to YAML: {}", e);
                                                continue;
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        eprintln!("Failed to create output file: {}", e);
                                        continue;
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("Error reading Postman collection {}: {}", path.display(), e);
                                continue;
                            }
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("Failed to read directory '{}': {}", cli.input_dir, e);
                process::exit(1);
            }
        }
        
        if files_processed == 0 {
            println!("No JSON files found in '{}'. Please add your Postman collections to this directory.", cli.input_dir);
        } else {
            println!("Processed {} collection(s).", files_processed);
        }
    }
}