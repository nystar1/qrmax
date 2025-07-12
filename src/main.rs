mod server;
mod tools;

use std::io::{BufRead, Write, stdin, stdout};

use serde_json::{Value, from_str, to_string};

use server::McpServer;
use tools::qr::{QrDecoderTool, QrGeneratorTool};

fn main() {
    let mut server = create_server();
    run_stdio_loop(&mut server);
}

fn create_server() -> McpServer {
    let mut server = McpServer::new();
    
    server.register_tool(Box::new(QrGeneratorTool));
    server.register_tool(Box::new(QrDecoderTool));
    
    server
}

fn run_stdio_loop(server: &McpServer) {
    let stdin = stdin();
    let mut stdout = stdout();
    
    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            if let Some(response) = process_line(server, &line) {
                if let Ok(response_json) = to_string(&response) {
                    writeln!(stdout, "{}", response_json).unwrap();
                    stdout.flush().unwrap();
                }
            }
        }
    }
}

fn process_line(server: &McpServer, line: &str) -> Option<Value> {
    let value = from_str::<Value>(line).ok()?;
    
    if value.get("id").is_some() {
        Some(server.handle_request(&value))
    } else {
        None
    }
}