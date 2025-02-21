use std::io::Read;
use std::path::Path;
use std::fs::File;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ConversionError;

#[derive(Debug, Deserialize, Serialize)]
pub struct PostmanCollection {
    pub info: Option<Info>,
    pub item: Option<Vec<Item>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Item {
    pub name: Option<String>,
    pub item: Option<Vec<Item>>,
    pub request: Option<Request>,
    pub response: Option<Vec<Response>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Request {
    pub method: Option<String>,
    pub url: Option<Url>,
    pub header: Option<Vec<Header>>,
    pub body: Option<Body>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Url {
    pub path: Option<Vec<Value>>,
    pub variable: Option<Vec<Variable>>,
    pub query: Option<Vec<Query>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Variable {
    pub key: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Query {
    pub key: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
    pub disabled: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Header {
    pub key: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
    pub disabled: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    pub mode: Option<String>,
    pub raw: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    pub name: Option<String>,
    pub code: Option<u32>,
    pub header: Option<Vec<Header>>,
    pub body: Option<String>,
}

impl PostmanCollection {
    pub fn parse(postman_file: &Path) -> Result<Self, ConversionError> {
        let mut file = File::open(postman_file)
            .map_err(ConversionError::FileError)?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(ConversionError::FileError)?;
        
        let collection: PostmanCollection = serde_json::from_str(&contents)
            .map_err(ConversionError::ParseError)?;
        
        if collection.info.is_none() || collection.item.is_none() {
            return Err(ConversionError::InvalidFormat(
                "Invalid Postman collection format".to_string()
            ));
        }
        
        Ok(collection)
    }
}

pub fn parse_postman_collection(postman_file: &Path) -> Result<PostmanCollection, ConversionError> {
    PostmanCollection::parse(postman_file)
}