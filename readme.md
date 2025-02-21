# POSTMAN to SWAGGER 

## Steps:

1. Put your collection in the folder named collections.
2. Run the Program.
3. The output will show up in output folder. 

## Run the Program with this:

### Convert all collections in the default folders:

python postman_to_swagger.py

### Convert a specific collection using default folders:

python postman_to_swagger.py --input "Warehouse API.postman_collection.json"

### Convert using custom folder names:

python postman_to_swagger.py --input-dir "my_collections" --output-dir "specs"

### Convert a specific collection with a custom output filename:

python postman_to_openapi.py --input "Warehouse API.postman_collection.json" --output "warehouse_spec.yaml"

### Fully custom path configuration:

python postman_to_openapi.py --input "Warehouse API.postman_collection.json" --output "warehouse_spec.yaml" --input-dir "api_files" --output-dir "yaml_files"