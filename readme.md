# POSTMAN to OpenAPI Converter

This tool converts Postman Collections to OpenAPI 3.0 YAML specifications. It automatically processes collection files and generates structured API documentation.

## 🚀 Features

- Converts Postman Collections to OpenAPI 3.0 YAML
- Preserves folder structure as tags
- Generates request/response schemas automatically
- Handles path parameters, query parameters, and headers
- Supports both single file and batch processing

## 📋 Prerequisites

- Python 3.6 or higher
- PyYAML package (`pip install pyyaml`)

## 🔧 Installation

1. Clone this repository or download the script
2. Install required packages:
   ```
   pip install pyyaml
   ```

## 📁 Directory Structure

```
├── collections/        # Place your Postman collections here
├── output/             # Generated OpenAPI specs will be saved here
├── postman_to_openapi.py  # The conversion script
└── README.md
```

## 🏃‍♂️ Usage

### Basic Usage: Convert All Collections

Process all JSON files in the collections folder:

```bash
python postman_to_openapi.py
```

### Convert a Specific Collection

```bash
python postman_to_openapi.py --input "Your API.postman_collection.json"
```

### Use Custom Directory Names

```bash
python postman_to_openapi.py --input-dir "my_collections" --output-dir "specs"
```

### Specify Custom Output Filename

```bash
python postman_to_openapi.py --input "Your API.postman_collection.json" --output "custom_name.yaml"
```

### Full Custom Configuration

```bash
python postman_to_openapi.py --input "Your API.postman_collection.json" --output "custom_name.yaml" --input-dir "api_files" --output-dir "yaml_files"
```

## 📝 Command-Line Arguments

| Argument | Description | Default |
|----------|-------------|---------|
| `--input` | Specific Postman collection filename | Process all JSON files |
| `--output` | Custom output filename | Based on input filename |
| `--input-dir` | Input directory for collections | "collections" |
| `--output-dir` | Output directory for specifications | "output" |

## 📤 Workflow

1. Place your Postman Collection JSON file(s) in the `collections` folder
2. Run the script with desired options
3. Find your OpenAPI specification(s) in the `output` folder

## 🔍 Example

Input: `collections/API.postman_collection.json`  
Command: `python postman_to_openapi.py`  
Output: `output/API_openapi.yaml`

## 🛠️ Troubleshooting

- Ensure your Postman Collection is in valid JSON format
- For large collections, check for any malformed requests or responses
- If you encounter errors, try converting a specific collection with the `--input` flag

## 📜 License

This project is licensed under the MIT License - see the LICENSE file for details.