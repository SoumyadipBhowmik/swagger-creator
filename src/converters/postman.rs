use crate::models::postman::*;
use crate::models::openapi::*;
use crate::utils::schema::infer_schema;
use std::collections::{HashMap, HashSet};
use indexmap::IndexMap;
use serde_json::{self, Value};

fn process_request_body(request: &Request) -> Option<RequestBody> {
    let body = match &request.body {
        Some(b) => b,
        None => return None,
    };
    
    if body.mode.as_deref() != Some("raw") {
        return None;
    }
    
    // Find content type header
    let content_type = request.header.as_ref()
        .and_then(|headers| {
            headers.iter()
                .find(|h| h.key.as_deref().map(|k| k.to_lowercase()) == Some("content-type".to_string()))
                .and_then(|h| h.value.clone())
        })
        .unwrap_or_else(|| "application/json".to_string());
    
    let raw = body.raw.as_deref().unwrap_or("{}");
    
    let mut content = HashMap::new();
    
    if content_type == "application/json" {
        match serde_json::from_str::<Value>(raw) {
            Ok(example) => {
                let schema = infer_schema(&example);
                content.insert(content_type, Content {
                    schema,
                    example: Some(example),
                    examples: None,
                });
            },
            Err(_) => return None,
        }
    } else {
        content.insert(content_type, Content {
            schema: Schema {
                schema_type: "string".to_string(),
                properties: None,
                required: None,
                items: None,
            },
            example: Some(Value::String(raw.to_string())),
            examples: None,
        });
    }
    
    Some(RequestBody {
        content,
        required: true,
    })
}

fn process_response(response: &Response) -> OpenAPIResponse {
    let _status_code = response.code.unwrap_or(200);
    
    // Find content type header
    let content_type = response.header.as_ref()
        .and_then(|headers| {
            headers.iter()
                .find(|h| h.key.as_deref().map(|k| k.to_lowercase()) == Some("content-type".to_string()))
                .and_then(|h| h.value.clone())
        })
        .unwrap_or_else(|| "application/json".to_string());
    
    let body = response.body.as_deref().unwrap_or("{}");
    
    let (schema, example) = if content_type == "application/json" {
        match serde_json::from_str::<Value>(body) {
            Ok(parsed) => (infer_schema(&parsed), parsed),
            Err(_) => {
                (
                    Schema {
                        schema_type: "string".to_string(),
                        properties: None,
                        required: None,
                        items: None,
                    },
                    Value::String(body.to_string()),
                )
            }
        }
    } else {
        (
            Schema {
                schema_type: "string".to_string(),
                properties: None,
                required: None,
                items: None,
            },
            Value::String(body.to_string()),
        )
    };
    
    let example_name = response.name.as_deref()
        .unwrap_or("example")
        .to_lowercase()
        .replace(" ", "_");
    
    let summary = response.name.clone().unwrap_or_else(|| "Example response".to_string());
    
    let mut examples = HashMap::new();
    examples.insert(example_name, Example {
        value: example.clone(),
        summary,
    });
    
    let mut content = HashMap::new();
    content.insert(content_type, Content {
        schema,
        example: None,
        examples: Some(examples),
    });
    
    OpenAPIResponse {
        description: response.name.clone().unwrap_or_else(|| "Response".to_string()),
        content,
    }
}

fn process_parameters(url_obj: &Url, headers: &Option<Vec<Header>>) -> Vec<Parameter> {
    let mut parameters = Vec::new();
    
    // Path parameters
    if let Some(variables) = &url_obj.variable {
        for var in variables {
            if let Some(key) = &var.key {
                parameters.push(Parameter {
                    name: key.clone(),
                    param_in: "path".to_string(),
                    schema: Schema {
                        schema_type: "string".to_string(),
                        properties: None,
                        required: None,
                        items: None,
                    },
                    description: None,
                    required: Some(true),
                });
            }
        }
    }
    
    // Query parameters
    if let Some(queries) = &url_obj.query {
        for query in queries {
            if let Some(key) = &query.key {
                parameters.push(Parameter {
                    name: key.clone(),
                    param_in: "query".to_string(),
                    schema: Schema {
                        schema_type: "string".to_string(),
                        properties: None,
                        required: None,
                        items: None,
                    },
                    description: query.description.clone(),
                    required: Some(query.disabled.unwrap_or(false) == false),
                });
            }
        }
    }
    
    // Headers
    if let Some(header_list) = headers {
        for header in header_list {
            if let Some(key) = &header.key {
                if key.to_lowercase() != "content-type" {
                    parameters.push(Parameter {
                        name: key.clone(),
                        param_in: "header".to_string(),
                        schema: Schema {
                            schema_type: "string".to_string(),
                            properties: None,
                            required: None,
                            items: None,
                        },
                        description: header.description.clone(),
                        required: Some(header.disabled.unwrap_or(false) == false),
                    });
                }
            }
        }
    }
    
    parameters
}

pub fn convert_postman_to_openapi(postman_collection: &PostmanCollection) -> OpenAPISpec {
    let mut openapi = OpenAPISpec {
        openapi: "3.0.0".to_string(),
        info: OpenAPIInfo {
            title: postman_collection.info.as_ref()
                .and_then(|info| info.name.clone())
                .unwrap_or_else(|| "API Documentation".to_string()),
            description: postman_collection.info.as_ref()
                .and_then(|info| info.description.clone())
                .unwrap_or_else(|| "".to_string()),
            version: "1.0.0".to_string(),
        },
        paths: IndexMap::new(),
        components: Components {
            schemas: IndexMap::new(),
        },
        tags: Vec::new(),
    };
    
    let mut tags_set = HashSet::new();
    
    fn process_item(
        item: &Item,
        current_tags: &[String],
        paths: &mut IndexMap<String, IndexMap<String, Operation>>,
        tags_set: &mut HashSet<String>,
    ) {
        if item.request.is_none() {
            return;
        }
        
        let request = item.request.as_ref().unwrap();
        let method = request.method.as_deref().unwrap_or("GET").to_lowercase();
        
        if let Some(url_obj) = &request.url {
            // Build path with {param} syntax
            let mut path_components = Vec::new();
            
            if let Some(path_parts) = &url_obj.path {
                for component in path_parts {
                    match component {
                        Value::Object(obj) => {
                            if let Some(Value::String(value)) = obj.get("value") {
                                path_components.push(format!("{{{}}}", value));
                            }
                        },
                        Value::String(s) => {
                            if s.starts_with(':') {
                                path_components.push(format!("{{{}}}", &s[1..]));
                            } else {
                                path_components.push(s.clone());
                            }
                        },
                        _ => {}
                    }
                }
            }
            
            let path = format!("/{}", path_components.join("/").trim_start_matches('/'));
            
            if !paths.contains_key(&path) {
                paths.insert(path.clone(), IndexMap::new());
            }
            
            let parameters = process_parameters(url_obj, &request.header);
            let request_body = if method != "get" && method != "delete" {
                process_request_body(request)
            } else {
                None
            };
            
            // Process responses
            let mut responses: IndexMap<String, OpenAPIResponse> = IndexMap::new();
            if let Some(response_list) = &item.response {
                for response in response_list {
                    let status_code = response.code.map(|c| c.to_string()).unwrap_or_else(|| "200".to_string());
                    let processed_resp = process_response(response);
                    
                    if responses.contains_key(&status_code) {
                        // Merge examples if same status code
                        if let Some(existing) = responses.get_mut(&status_code) {
                            for (content_type, content) in &processed_resp.content {
                                if let Some(existing_content) = existing.content.get_mut(content_type) {
                                    if let Some(new_examples) = &content.examples {
                                        if let Some(existing_examples) = &mut existing_content.examples {
                                            for (key, example) in new_examples {
                                                existing_examples.insert(key.clone(), example.clone());
                                            }
                                        } else {
                                            existing_content.examples = content.examples.clone();
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        responses.insert(status_code, processed_resp);
                    }
                }
            }
            
            // If no responses, add a default 200 OK
            if responses.is_empty() {
                responses.insert("200".to_string(), OpenAPIResponse {
                    description: "OK".to_string(),
                    content: HashMap::new(),
                });
            }
            
            // Add tags to the tag set
            for tag in current_tags {
                tags_set.insert(tag.clone());
            }
            
            // Build the operation
            let operation = Operation {
                summary: item.name.clone().unwrap_or_else(|| "".to_string()),
                description: request.description.clone().unwrap_or_else(|| "".to_string()),
                parameters,
                request_body,
                responses,
                tags: current_tags.to_vec(),
            };
            
            if let Some(path_map) = paths.get_mut(&path) {
                path_map.insert(method, operation);
            }
        }
    }
    
    fn process_items(
        items: &[Item],
        current_tags: &[String],
        paths: &mut IndexMap<String, IndexMap<String, Operation>>, 
        tags_set: &mut HashSet<String>,
    ) {
        for item in items {
            if let Some(subitems) = &item.item {
                let folder_name = item.name.clone().unwrap_or_else(|| "".to_string());
                let mut new_tags = current_tags.to_vec();
                if !folder_name.is_empty() {
                    new_tags.push(folder_name);
                }
                process_items(subitems, &new_tags, paths, tags_set);
            } else {
                process_item(item, current_tags, paths, tags_set);
            }
        }
    }
    
    if let Some(items) = &postman_collection.item {
        process_items(items, &[], &mut openapi.paths, &mut tags_set);
    }
    
    // Convert tags set to vector
    openapi.tags = tags_set.into_iter()
        .map(|tag| Tag { name: tag })
        .collect();
    
    // Sort tags alphabetically
    openapi.tags.sort_by(|a, b| a.name.cmp(&b.name));
    
    openapi
}