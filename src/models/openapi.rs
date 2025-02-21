use indexmap::IndexMap;
use serde::Serialize;
use std::collections::HashMap;
use serde_json::Value;


#[derive(Debug, Serialize)]
pub struct OpenAPISpec {
    pub openapi: String,
    pub info: OpenAPIInfo,
    pub paths: IndexMap<String, IndexMap<String, Operation>>,
    pub components: Components,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Serialize)]
pub struct OpenAPIInfo {
    pub title: String,
    pub description: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct Components {
    pub schemas: IndexMap<String, Value>,
}

#[derive(Debug, Serialize)]
pub struct Tag {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct Operation {
    pub summary: String,
    pub description: String,
    pub parameters: Vec<Parameter>,
    #[serde(rename = "requestBody", skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,
    pub responses: IndexMap<String, OpenAPIResponse>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub param_in: String,
    pub schema: Schema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Schema {
    #[serde(rename = "type")]
    pub schema_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
}

#[derive(Debug, Serialize)]
pub struct RequestBody {
    pub content: HashMap<String, Content>,
    pub required: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct Content {
    pub schema: Schema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<HashMap<String, Example>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Example {
    pub value: Value,
    pub summary: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct OpenAPIResponse {
    pub description: String,
    pub content: HashMap<String, Content>,
}