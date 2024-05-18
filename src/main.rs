use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{self, Read, Write};

#[derive(Debug, Deserialize)]
struct Swagger {
    info: HashMap<String, Value>,
    definitions: HashMap<String, Definition>,
    paths: HashMap<String, PathItem>,
    schemes: Option<Vec<String>>,
    host: Option<String>,
    basePath: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Definition {
    #[serde(rename = "type")]
    definition_type: Option<String>,
    properties: Option<HashMap<String, Property>>,
    required: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Property {
    #[serde(rename = "type")]
    property_type: Option<String>,
    format: Option<String>,
    #[serde(flatten)]
    additional: HashMap<String, Value>,
    reference: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PathItem {
    get: Option<Operation>,
    post: Option<Operation>,
    put: Option<Operation>,
    delete: Option<Operation>,
}

#[derive(Debug, Deserialize)]
struct Operation {
    operation_id: Option<String>,
    summary: Option<String>,
    responses: HashMap<String, Response>,
}

#[derive(Debug, Deserialize)]
struct Response {
    description: String,
    #[serde(rename = "schema")]
    response_schema: Option<Schema>,
}

#[derive(Debug, Deserialize)]
struct Schema {
    #[serde(rename = "type")]
    schema_type: Option<String>,
    #[serde(rename = "$ref")]
    reference: Option<String>,
}

fn main() -> io::Result<()> {
    let mut file = File::open("swagger.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let swagger: Swagger = serde_json::from_str(&data).expect("Invalid JSON");

    create_dir_all("output/interfaces")?;

    for (name, definition) in &swagger.definitions {
        let ts_interface = generate_typescript_interface(&swagger, name, definition);
        let mut file = File::create(format!("output/interfaces/{}.ts", name))?;
        file.write_all(ts_interface.as_bytes())?;
    }

    write_service(&swagger, "typescript", "output/service.ts")?;
    // write_service(&swagger, "javascript", "output/service.js")?;
    // write_service(&swagger, "python", "output/service.py")?;
    // write_service(&swagger, "go", "output/service.go")?;
    // write_service(&swagger, "rust", "output/service.rs")?;
    // write_service(&swagger, "java", "output/Service.java")?;
    // write_service(&swagger, "csharp", "output/Service.cs")?;

    Ok(())
}

fn write_service(swagger: &Swagger, language: &str, filename: &str) -> std::io::Result<()> {
    let service = generate_service(swagger, language);
    let mut file = File::create(filename)?;
    file.write_all(service.as_bytes())?;
    Ok(())
}

fn generate_typescript_interface(swagger: &Swagger, name: &str, definition: &Definition) -> String {
    let mut ts_code = String::new();
    generate_info_comment(swagger, &mut ts_code);
    ts_code.push_str("export interface ");
    ts_code.push_str(name);
    ts_code.push_str(" {\n");

    if let Some(properties) = &definition.properties {
        for (prop_name, prop) in properties {
            let ts_type = match prop.property_type.as_deref() {
                Some("integer") => "number",
                Some("string") => "string",
                Some("boolean") => "boolean",
                Some("array") => {
                    let items = &prop.additional["items"];
                    if let Some(item_type) = items.get("type").and_then(Value::as_str) {
                        match item_type {
                            "integer" => "number[]",
                            "string" => "string[]",
                            "boolean" => "boolean[]",
                            _ => "any[]",
                        }
                    } else {
                        "any[]"
                    }
                }
                Some("object") => {
                    if let Some(ref_name) = prop.reference.as_deref() {
                        ref_name
                    } else {
                        "any"
                    }
                }
                _ => "any",
            };
            let optional = if definition
                .required
                .as_ref()
                .map_or(false, |r| !r.contains(prop_name))
            {
                "?"
            } else {
                ""
            };
            ts_code.push_str(&format!("    {}{}: {};\n", prop_name, optional, ts_type));
        }
    }
    ts_code.push_str("}\n");
    ts_code
}

fn generate_info_comment(swagger: &Swagger, ts_code: &mut String) {
    let generated_date = chrono::Local::now().format("%Y-%m-%d").to_string();
    ts_code.push_str("/*\n");
    ts_code.push_str(" * This file was generated by swagger-genereator\n");
    ts_code.push_str(" * Do not modify this file manually.\n");
    ts_code.push_str(" * Version: "); ts_code.push_str(&swagger.info["version"].as_str().unwrap());
    ts_code.push_str("\n");
    ts_code.push_str(" * Title: "); ts_code.push_str(&swagger.info["title"].as_str().unwrap());
    ts_code.push_str("\n");
    ts_code.push_str(" * Description: "); ts_code.push_str(&swagger.info["description"].as_str().unwrap());
    ts_code.push_str("\n");
    ts_code.push_str(" * Author: Muhtalip Dede\n");
    ts_code.push_str(" * Generated on: "); ts_code.push_str(&generated_date);
    ts_code.push_str(" */\n\n");
}

fn generate_service(swagger: &Swagger, lang: &str) -> String {
    let mut ts_code = String::new();

    generate_info_comment(swagger, &mut ts_code);

    ts_code.push_str("import axios from 'axios';\n\n");
    ts_code.push_str("axios.defaults.baseURL = '");
    ts_code.push_str(&swagger.schemes.as_ref().unwrap()[0]);
    ts_code.push_str("://");
    ts_code.push_str(&swagger.host.as_ref().unwrap());
    ts_code.push_str(&swagger.basePath.as_ref().unwrap());
    ts_code.push_str("';\n\n");

    if lang == "typescript" {
        let interfaces = std::fs::read_dir("output/interfaces").unwrap();

        for interface in interfaces {
            let interface = interface.unwrap();
            let interface_name = interface.file_name().into_string().unwrap().replace(".ts", "");
            ts_code.push_str(&format!("import {{ {} }} from './interfaces/{}';\n", interface_name, interface_name));
        }
        ts_code.push_str("\n");
    }

    for (path, path_item) in &swagger.paths {
        if let Some(operation) = &path_item.get {
            ts_code.push_str(&generate_service_method("get", path, operation, lang));
        }
        if let Some(operation) = &path_item.post {
            ts_code.push_str(&generate_service_method("post", path, operation, lang));
        }
        if let Some(operation) = &path_item.put {
            ts_code.push_str(&generate_service_method("put", path, operation, lang));
        }
        if let Some(operation) = &path_item.delete {
            ts_code.push_str(&generate_service_method("delete", path, operation, lang));
        }
    }

    ts_code
}

fn generate_service_method(method: &str, path: &str, operation: &Operation, lang: &str) -> String {
    let operation_id = operation
        .operation_id
        .as_deref()
        .unwrap_or("unknown")
        .to_string();
    let fallback_operation_id = path
        .split('/')
        .filter(|s| !s.is_empty() && !s.starts_with('{'))
        .collect::<Vec<&str>>()
        .join("_");
    let final_operation_id = if operation_id == "unknown" {
        fallback_operation_id
    } else {
        operation_id
    };

    let path_params = extract_path_params(path);
    let params_declaration = if path_params.is_empty() {
        "".to_string()
    } else {
        path_params
            .iter()
            .map(|param| format!("{}: string", param))
            .collect::<Vec<String>>()
            .join(", ")
            + ", "
    };

    let data_param = if method == "get" || method == "delete" {
        ""
    } else {
        "data?: any, "
    };

    let formatted_path = path_params.iter().fold(path.to_string(), |acc, param| {
        acc.replace(&format!("{{{}}}", param), &format!("${{{}}}", param))
    });

    let mut method_name = method.to_lowercase()
        + final_operation_id
            .split('_')
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<String>()
            .as_str();

    if !params_declaration.is_empty() {
        method_name = format!("{}ById", method_name);
    }

    let mut response_schema = operation
        .responses
        .get("200")
        .and_then(|r| r.response_schema.as_ref())
        .and_then(|s| s.reference.as_ref())
        .map_or_else(|| "any".to_string(), |r| r.to_string());

    if response_schema.starts_with("#/definitions/") {
        response_schema = response_schema.replace("#/definitions/", "");
    }

    let response_type = if lang == "typescript" {
        format!("Promise<{}>", response_schema)
    } else {
        "Promise<any>".to_string()
    };

    let method_code = format!(
        "export async function {}({}{}config?: any): {} {{
    const response = await axios.{}(`{}`, {}config);
    return response.data;
}}\n\n",
        method_name,
        params_declaration,
        data_param,
        response_type,
        method,
        formatted_path,
        if data_param.is_empty() { "" } else { "data, " }
    );

    method_code
}

fn extract_path_params(path: &str) -> Vec<String> {
    let mut params = Vec::new();
    for segment in path.split('/') {
        if segment.starts_with('{') && segment.ends_with('}') {
            params.push(segment[1..segment.len() - 1].to_string());
        }
    }
    params
}