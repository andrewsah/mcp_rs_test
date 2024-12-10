use log::LevelFilter;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use simplelog::{Config, WriteLogger};
use std::{
    fmt::Display,
    fs::File,
    io::{self, BufRead, Write},
};

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

fn handle_request(request: &JsonRpcRequest) {
    log::info!("handle_request: {:?}", request);
    match request.method.as_str() {
        "initialize" => {
            log::info!("Initializing server...");
            let mut result = Value::Object(Default::default());
            result["protocolVersion"] = request.params.as_ref().unwrap()["protocolVersion"].clone();
            result["capabilities"] = Value::Object(Default::default());
            result["serverInfo"] = Value::Object(Default::default());
            result["serverInfo"]["name"] = Value::String("MCP Rust test server".to_string());
            result["serverInfo"]["version"] = Value::String("0.1.0".to_string());
            let response = JsonRpcResponseSuccess {
                id: request.id.clone(),
                jsonrpc: "2.0".to_string(),
                result: Some(result),
            };
            let response_str = serde_json::to_string(&response);
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
        _ => {
            log::error!("Unknown method: {}", request.method);
        }
    }
}

fn handle_notification(notification: &JsonRpcNotification) {
    log::info!("handle_notification: {:?}", notification);
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
                match notification {
                    Ok(notif) => handle_notification(&notif),
                    Err(e) => log::error!("Error parsing notification: {}", e),
                }
            }
        }
    }
}
