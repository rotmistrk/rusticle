//! LSP server — dispatch and document state.

mod diagnostics;
mod features;
mod preamble;
mod util;

use std::collections::HashMap;

use crate::interpreter::Interpreter;
use serde_json::{json, Value};

pub struct Server {
    pub(crate) interp: Interpreter,
    pub(crate) documents: HashMap<String, String>,
    /// Cache: interpreter name → preamble text (empty = failed/no preamble).
    preamble_cache: HashMap<String, String>,
    exit: bool,
}

impl Server {
    pub fn new(interp: Interpreter) -> Self {
        Self {
            interp,
            documents: HashMap::new(),
            preamble_cache: HashMap::new(),
            exit: false,
        }
    }

    pub fn should_exit(&self) -> bool {
        self.exit
    }

    /// Handle a JSON-RPC message, returning zero or more responses.
    pub fn handle(&mut self, msg: &Value) -> Vec<Value> {
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
        let id = msg.get("id");

        match method {
            "initialize" => vec![self.initialize(id)],
            "initialized" => vec![],
            "shutdown" => vec![self.shutdown(id)],
            "exit" => {
                self.exit = true;
                vec![]
            }
            "textDocument/didOpen" => self.did_open(msg),
            "textDocument/didChange" => self.did_change(msg),
            "textDocument/didClose" => self.did_close(msg),
            "textDocument/completion" => vec![features::completion(self, id, msg)],
            "textDocument/hover" => vec![features::hover(self, id, msg)],
            "textDocument/definition" => vec![features::definition(self, id, msg)],
            "textDocument/references" => vec![features::references(self, id, msg)],
            "textDocument/rename" => {
                vec![json!({"jsonrpc": "2.0", "id": id, "result": {"changes": {}}})]
            }
            "textDocument/codeAction" => vec![json!({"jsonrpc": "2.0", "id": id, "result": []})],
            _ => {
                if let Some(id) = id {
                    vec![json!({
                        "jsonrpc": "2.0", "id": id,
                        "error": {"code": -32601, "message": format!("Method not found: {method}")}
                    })]
                } else {
                    vec![]
                }
            }
        }
    }

    fn initialize(&self, id: Option<&Value>) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "capabilities": {
                    "textDocumentSync": 1,
                    "completionProvider": {"triggerCharacters": ["$", " "]},
                    "hoverProvider": true,
                    "definitionProvider": true,
                    "referencesProvider": true,
                    "renameProvider": true,
                    "codeActionProvider": true
                }
            }
        })
    }

    fn shutdown(&self, id: Option<&Value>) -> Value {
        json!({"jsonrpc": "2.0", "id": id, "result": null})
    }

    fn did_open(&mut self, msg: &Value) -> Vec<Value> {
        let params = &msg["params"]["textDocument"];
        let uri = params["uri"].as_str().unwrap_or("");
        let text = params["text"].as_str().unwrap_or("");
        self.documents.insert(uri.to_string(), text.to_string());
        self.load_shebang_preamble(text);
        vec![diagnostics::publish(self, uri)]
    }

    fn did_change(&mut self, msg: &Value) -> Vec<Value> {
        let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("");
        if let Some(change) = msg["params"]["contentChanges"]
            .as_array()
            .and_then(|a| a.first())
        {
            if let Some(text) = change["text"].as_str() {
                self.documents.insert(uri.to_string(), text.to_string());
            }
        }
        vec![diagnostics::publish(self, uri)]
    }

    fn did_close(&mut self, msg: &Value) -> Vec<Value> {
        let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("");
        self.documents.remove(uri);
        vec![json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": {"uri": uri, "diagnostics": []}
        })]
    }

    /// Parse shebang from text, exec interpreter --lsp-preamble, cache and eval result.
    fn load_shebang_preamble(&mut self, text: &str) {
        let interpreter = match preamble::parse_shebang(text) {
            Some(name) => name,
            None => return,
        };
        if self.preamble_cache.contains_key(&interpreter) {
            return;
        }
        let output = preamble::exec_lsp_preamble(&interpreter);
        if !output.is_empty() {
            let _ = self.interp.eval(&output);
        }
        self.preamble_cache.insert(interpreter, output);
    }
}
