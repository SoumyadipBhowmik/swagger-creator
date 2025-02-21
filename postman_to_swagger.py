import json
import yaml
import argparse
from urllib.parse import urlparse
import sys
import os
from typing import Dict, List, Any, Optional

def parse_postman_collection(postman_file: str) -> Dict:
    try:
        with open(postman_file, 'r') as f:
            collection = json.load(f)
            if 'info' not in collection or 'item' not in collection:
                raise ValueError("Invalid Postman collection format")
            return collection
    except Exception as e:
        print(f"Error reading Postman collection: {str(e)}")
        sys.exit(1)

def process_request_body(request: Dict) -> Optional[Dict]:
    if 'body' not in request:
        return None

    body = request['body']
    if body.get('mode') != 'raw':
        return None

    content_type = next((h['value'] for h in request.get('header', [])
                       if h.get('key', '').lower() == 'content-type'), 'application/json')

    try:
        if content_type == 'application/json':
            example = json.loads(body.get('raw', '{}'))
            schema = infer_schema(example)
        else:
            example = body.get('raw', '')
            schema = {"type": "string"}

        return {
            "content": {
                content_type: {
                    "schema": schema,
                    "example": example
                }
            },
            "required": True
        }
    except json.JSONDecodeError:
        return None

def process_response(response: Dict) -> Dict:
    status_code = str(response.get('code', 200))
    headers = response.get('header', []) or []
    content_type = next((h['value'] for h in headers
                   if h.get('key', '').lower() == 'content-type'), 'application/json')

    try:
        example = json.loads(response.get('body', '{}')) if content_type == 'application/json' else response.get('body', '')
        schema = infer_schema(example) if content_type == 'application/json' else {"type": "string"}
    except json.JSONDecodeError:
        example = response.get('body', '')
        schema = {"type": "string"}

    return {
        "description": response.get('name', 'Response'),
        "content": {
            content_type: {
                "schema": schema,
                "examples": {
                    response.get('name', 'example').lower().replace(' ', '_'): {
                        "value": example,
                        "summary": response.get('name', 'Example response')
                    }
                }
            }
        }
    }

def infer_schema(data: Any) -> Dict:
    if isinstance(data, dict):
        properties = {}
        required = []
        for key, value in data.items():
            properties[key] = infer_schema(value)
            if value is not None:
                required.append(key)
        return {
            "type": "object",
            "properties": properties,
            "required": required
        }
    elif isinstance(data, list):
        return {"type": "array", "items": infer_schema(data[0])} if data else {"type": "array", "items": {}}
    elif isinstance(data, bool):
        return {"type": "boolean"}
    elif isinstance(data, (int, float)):
        return {"type": "number"}
    elif isinstance(data, str):
        return {"type": "string"}
    else:
        return {"type": "string"}

def process_parameters(url_obj: Dict, headers: List[Dict]) -> List[Dict]:
    parameters = []

    # Path parameters from Postman URL variables
    for var in url_obj.get('variable', []):
        parameters.append({
            "name": var.get('key'),
            "in": "path",
            "required": True,
            "schema": {"type": "string"}
        })

    # Query parameters
    for query in url_obj.get('query', []):
        parameters.append({
            "name": query.get('key'),
            "in": "query",
            "schema": {"type": "string"},
            "description": query.get('description', ''),
            "required": query.get('disabled', False) is False
        })

    # Headers
    for header in headers:
        if header.get('key', '').lower() != 'content-type':
            parameters.append({
                "name": header.get('key'),
                "in": "header",
                "schema": {"type": "string"},
                "description": header.get('description', ''),
                "required": header.get('disabled', False) is False
            })

    return parameters

def convert_postman_to_openapi(postman_collection: Dict) -> Dict:
    openapi = {
        "openapi": "3.0.0",
        "info": {
            "title": postman_collection.get('info', {}).get('name', 'API Documentation'),
            "description": postman_collection.get('info', {}).get('description', ''),
            "version": "1.0.0"
        },
        "paths": {},
        "components": {"schemas": {}},
        "tags": []
    }

    tags_set = set()

    def process_item(item: Dict, current_tags: List[str]) -> None:
        nonlocal tags_set
        if 'request' not in item:
            return

        request = item['request']
        method = request.get('method', 'GET').lower()
        url_obj = request.get('url', {})

        # Build path with {param} syntax
        path_components = []
        for component in url_obj.get('path', []):
            if isinstance(component, dict):
                path_components.append(f"{{{component.get('value', '')}}}")
            else:
                if component.startswith(':'):
                    path_components.append(f"{{{component[1:]}}}")
                else:
                    path_components.append(component)
        path = '/' + '/'.join(path_components).lstrip('/')

        if path not in openapi['paths']:
            openapi['paths'][path] = {}

        parameters = process_parameters(url_obj, request.get('header', []))
        request_body = None if method in ['get', 'delete'] else process_request_body(request)

        # Process responses
        responses = {}
        for response in item.get('response', []):
            status_code = str(response.get('code', 200))
            processed_resp = process_response(response)
            
            if status_code in responses:
                existing = responses[status_code]
                content_type = next(iter(processed_resp['content'].keys()), None)
                if content_type:
                    existing['content'][content_type]['examples'].update(
                        processed_resp['content'][content_type]['examples']
                    )
            else:
                responses[status_code] = processed_resp

        operation_tags = current_tags.copy()
        for tag in operation_tags:
            tags_set.add(tag)

        operation = {  # Build the operation dictionary
            "summary": item.get('name', ''),
            "description": request.get('description', ''),
            "parameters": parameters,
            "responses": responses or {"200": {"description": "OK"}},
            "tags": operation_tags
        }

        if request_body:  # Only add requestBody if it's not None
            operation["requestBody"] = request_body

        openapi['paths'][path][method] = operation

    def process_items(items: List[Dict], current_tags: List[str]) -> None:
        for item in items:
            if 'item' in item:
                folder_name = item.get('name', '')
                new_tags = current_tags + [folder_name]
                process_items(item['item'], new_tags)
            else:
                process_item(item, current_tags)

    process_items(postman_collection.get('item', []), [])

    openapi['tags'] = [{"name": tag} for tag in sorted(tags_set)]

    return openapi

def main():
    parser = argparse.ArgumentParser(description='Convert Postman Collection to OpenAPI YAML')
    parser.add_argument('--input', help='Input Postman collection filename', default=None)
    parser.add_argument('--output', help='Output OpenAPI YAML filename', default=None)
    parser.add_argument('--input-dir', help='Input directory for Postman collections', default='collections')
    parser.add_argument('--output-dir', help='Output directory for OpenAPI specs', default='output')
    args = parser.parse_args()

    # Create output directory if it doesn't exist
    if not os.path.exists(args.output_dir):
        os.makedirs(args.output_dir)
        print(f"Created output directory: {args.output_dir}")
    
    # Process a single file if specified
    if args.input:
        input_path = os.path.join(args.input_dir, args.input)
        
        # Determine output filename based on input if not specified
        if args.output:
            output_filename = args.output
        else:
            base_name = os.path.splitext(os.path.basename(args.input))[0]
            output_filename = f"{base_name}_openapi.yaml"
        
        output_path = os.path.join(args.output_dir, output_filename)
        
        # Convert the collection
        print(f"Processing: {input_path}")
        postman_collection = parse_postman_collection(input_path)
        openapi_spec = convert_postman_to_openapi(postman_collection)
        
        with open(output_path, 'w') as f:
            yaml.dump(openapi_spec, f, sort_keys=False, allow_unicode=True)
        
        print(f"Converted to {output_path}")
    
    # Process all files in input directory if no specific input file
    else:
        if not os.path.exists(args.input_dir):
            print(f"Input directory '{args.input_dir}' does not exist. Please create it and add your Postman collections.")
            sys.exit(1)
            
        files_processed = 0
        for filename in os.listdir(args.input_dir):
            if filename.endswith('.json'):
                input_path = os.path.join(args.input_dir, filename)
                base_name = os.path.splitext(filename)[0]
                output_path = os.path.join(args.output_dir, f"{base_name}_openapi.yaml")
                
                print(f"Processing: {input_path}")
                postman_collection = parse_postman_collection(input_path)
                openapi_spec = convert_postman_to_openapi(postman_collection)
                
                with open(output_path, 'w') as f:
                    yaml.dump(openapi_spec, f, sort_keys=False, allow_unicode=True)
                
                print(f"Converted to {output_path}")
                files_processed += 1
        
        if files_processed == 0:
            print(f"No JSON files found in '{args.input_dir}'. Please add your Postman collections to this directory.")
        else:
            print(f"Processed {files_processed} collection(s).")

if __name__ == '__main__':
    main()