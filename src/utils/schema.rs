use std::collections::HashMap;

use serde_json::Value;
use crate::models::openapi::Schema;

pub fn infer_schema(data: &Value) -> Schema {
    match data {
        Value::Object(obj) => {
            let mut properties = HashMap::new();
            let mut required = Vec::new();
            
            for (key, value) in obj {
                properties.insert(key.clone(), infer_schema(value));
                if !value.is_null() {
                    required.push(key.clone());
                }
            }
            
            Schema {
                schema_type: "object".to_string(),
                properties: Some(properties),
                required: if required.is_empty() { None } else { Some(required) },
                items: None,
            }
        },
        Value::Array(arr) => {
            if let Some(first) = arr.first() {
                Schema {
                    schema_type: "array".to_string(),
                    properties: None,
                    required: None,
                    items: Some(Box::new(infer_schema(first))),
                }
            } else {
                Schema {
                    schema_type: "array".to_string(),
                    properties: None,
                    required: None,
                    items: Some(Box::new(Schema {
                        schema_type: "object".to_string(),
                        properties: None,
                        required: None,
                        items: None,
                    })),
                }
            }
        },
        Value::Bool(_) => Schema {
            schema_type: "boolean".to_string(),
            properties: None,
            required: None,
            items: None,
        },
        Value::Number(_) => Schema {
            schema_type: "number".to_string(),
            properties: None,
            required: None,
            items: None,
        },
        Value::String(_) => Schema {
            schema_type: "string".to_string(),
            properties: None,
            required: None,
            items: None,
        },
        _ => Schema {
            schema_type: "string".to_string(),
            properties: None,
            required: None,
            items: None,
        },
    }
}