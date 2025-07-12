use std::collections::HashMap;
use serde_json::{json, Value};

pub trait ToolHandler {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    fn execute(&self, params: Option<Value>) -> Result<Value, String>;
}

pub struct McpServer {
    tools: HashMap<String, Box<dyn ToolHandler>>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Box<dyn ToolHandler>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    pub fn handle_request(&self, request: &Value) -> Value {
        let method = request.get("method").and_then(|v| v.as_str());
        let id = request.get("id").cloned();
        
        match method {
            Some("initialize") => self.handle_initialize(id),
            Some("tools/list") => self.handle_tools_list(id),
            Some("tools/call") => self.handle_tools_call(id, request.get("params").cloned()),
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": "Method not found"
                }
            }),
        }
    }

    fn handle_initialize(&self, id: Option<Value>) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "qrmax",
                    "version": "0.1.0"
                }
            }
        })
    }

    fn handle_tools_list(&self, id: Option<Value>) -> Value {
        let tools: Vec<Value> = self.tools.values().map(|tool| {
            json!({
                "name": tool.name(),
                "description": tool.description(),
                "inputSchema": tool.input_schema()
            })
        }).collect();

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": tools
            }
        })
    }

    fn handle_tools_call(&self, id: Option<Value>, params: Option<Value>) -> Value {
        let params = match params {
            Some(p) => p,
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params"
                    }
                });
            }
        };

        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Missing tool name"
                    }
                });
            }
        };

        let tool_args = params.get("arguments");

        match self.tools.get(tool_name) {
            Some(tool) => {
                match tool.execute(tool_args.cloned()) {
                    Ok(result) => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                                }
                            ]
                        }
                    }),
                    Err(err) => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32000,
                            "message": err
                        }
                    }),
                }
            }
            None => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Tool '{}' not found", tool_name)
                }
            }),
        }
    }
}