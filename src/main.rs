use log::LevelFilter;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use simplelog::{Config, WriteLogger};
use std::{
    fmt::Display,
    fs::File,
    io::{self, BufRead, Write},
};

#[allow(dead_code)]
const ERROR_CODE_PARSE_ERROR: i32 = -32700;
#[allow(dead_code)]
const ERROR_CODE_INVALID_REQUEST: i32 = -32600;
#[allow(dead_code)]
const ERROR_CODE_METHOD_NOT_FOUND: i32 = -32601;
#[allow(dead_code)]
const ERROR_CODE_INVALID_PARAMS: i32 = -32602;
#[allow(dead_code)]
const ERROR_CODE_INTERNAL_ERROR: i32 = -32603;

/// Represents a JSON-RPC ID that can be either a number or string according to the JSON-RPC 2.0 specification
/// See https://www.jsonrpc.org/specification#id1
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum JsonRpcId {
    Number(u64),
    String(String),
}

impl JsonRpcId {
    fn clone(&self) -> JsonRpcId {
        match self {
            JsonRpcId::Number(n) => JsonRpcId::Number(*n),
            JsonRpcId::String(s) => JsonRpcId::String(s.clone()),
        }
    }
}

impl Display for JsonRpcId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonRpcId::Number(n) => write!(f, "{}", n),
            JsonRpcId::String(s) => write!(f, "{}", s),
        }
    }
}

/// Represents a JSON-RPC request object according to the JSON-RPC 2.0 specification.
/// See https://www.jsonrpc.org/specification
#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcRequest {
    /// Identifier established by the client that should be sent back in the response
    id: JsonRpcId,
    /// JSON-RPC protocol version, must be "2.0"
    jsonrpc: String,
    /// Name of the method to be invoked
    method: String,
    /// Parameters to pass to the method, if any
    params: Option<Value>,
}

/// Represents a JSON-RPC error object according to the JSON-RPC 2.0 specification.
/// See https://www.jsonrpc.org/specification
#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcError {
    /// A number indicating the error type that occurred
    code: i32,
    /// A short description of the error
    message: String,
    /// Additional information about the error, if available
    data: Option<Value>,
}

trait JsonRpcResponse {
    fn to_json(&self) -> Result<String, serde_json::Error>;
}

/// Represents a JSON-RPC response object according to the JSON-RPC 2.0 specification.
/// See https://www.jsonrpc.org/specification
#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponseSuccess {
    /// Identifier matching the id that was sent in the request
    id: JsonRpcId,
    /// JSON-RPC protocol version, must be "2.0"
    jsonrpc: String,
    /// Result of the RPC call if successful
    result: Option<Value>,
}

impl JsonRpcResponse for JsonRpcResponseSuccess {
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Represents a JSON-RPC response object according to the JSON-RPC 2.0 specification.
/// See https://www.jsonrpc.org/specification
#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponseError {
    /// Identifier matching the id that was sent in the request
    id: JsonRpcId,
    /// JSON-RPC protocol version, must be "2.0"
    jsonrpc: String,
    /// Error information if the RPC call failed
    error: Option<JsonRpcError>,
}

impl JsonRpcResponse for JsonRpcResponseError {
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Represents a JSON-RPC notification object according to the JSON-RPC 2.0 specification.
/// Notifications are similar to requests but do not require a response from the server.
/// See https://www.jsonrpc.org/specification
#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcNotification {
    /// JSON-RPC protocol version, must be "2.0"
    jsonrpc: String,
    /// Name of the method to be invoked
    method: String,
    /// Parameters to pass to the method, if any
    params: Option<Value>,
}

fn send_response<T: JsonRpcResponse>(response: T) {
    let response_str = response.to_json();
    match response_str {
        Ok(s) => {
            log::info!("Sending response: {}", s);
            let mut stdout = io::stdout();
            stdout.write_all(s.as_bytes()).unwrap();
            stdout.write_all(b"\n").unwrap();
            stdout.flush().unwrap();
        }
        Err(e) => log::error!("Error serializing response: {}", e),
    }
}

fn handle_request(request: &JsonRpcRequest) {
    log::info!("handle_request: {:?}", request);
    match request.method.as_str() {
        "initialize" => {
            log::info!("Initializing server...");
            let mut result = Value::Object(Default::default());
            result["protocolVersion"] = Value::String("2024-11-05".to_string());
            result["capabilities"] = Value::Object(Default::default());
            // result["capabilities"]["prompts"] = Value::Object(Default::default());
            // result["capabilities"]["prompts"]["listChanged"] = Value::Bool(true);
            result["serverInfo"] = Value::Object(Default::default());
            result["serverInfo"]["name"] = Value::String("MCP Rust test server".to_string());
            result["serverInfo"]["version"] = Value::String("0.1.0".to_string());
            let response = JsonRpcResponseSuccess {
                id: request.id.clone(),
                jsonrpc: "2.0".to_string(),
                result: Some(result),
            };
            send_response(response);
        }
        "ping" => {
            log::info!("Client ping server...");
            let response = JsonRpcResponseSuccess {
                id: request.id.clone(),
                jsonrpc: "2.0".to_string(),
                result: Some(Value::Object(Default::default())),
            };
            send_response(response);
        }
        _ => {
            log::error!("Unknown request method: {}", request.method);
            let err = JsonRpcError {
                code: ERROR_CODE_INVALID_REQUEST,
                message: format!("Invalid request: '{}'", request.method),
                data: None
            };
            let response = JsonRpcResponseError {
                id: request.id.clone(),
                jsonrpc: "2.0".to_string(),
                error: Some(err)
            };
            send_response(response);
        }
    }
}

fn handle_notification(notification: &JsonRpcNotification) {
    log::info!("handle_notification: {:?}", notification);
    match notification.method.as_str() {
        "notifications/initialized" => {
            log::info!("Server initialized.");
        }
        _ => {
            log::error!("Unknown notification method: {}", notification.method);
        }
    }
}

fn main() {
    // Initialize the logger to write to a file
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("C:\\tmp\\my_rust_bin.log").unwrap(),
    );

    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        if let Ok(input) = line {
            log::info!("Received line: {}", input);
            let request = serde_json::from_str::<JsonRpcRequest>(&input);
            if let Ok(req) = request {
                handle_request(&req);
            } else {
                let notification = serde_json::from_str::<JsonRpcNotification>(&input);
                if let Ok(notif) = notification {
                    handle_notification(&notif);
                } else {
                    log::error!("Error parsing request: {:?}", request);
                    let err = JsonRpcError {
                        code: ERROR_CODE_PARSE_ERROR,
                        message: "Parse error".to_string(),
                        data: None
                    };
                    let response = JsonRpcResponseError {
                        id: JsonRpcId::Number(0),
                        jsonrpc: "2.0".to_string(),
                        error: Some(err)
                    };
                    send_response(response);
                }
            }
        }
    }
}
