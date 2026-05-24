//! LSP language features: completion, hover, definition, references.

use crate::parser::Parser;
use serde_json::{json, Value};

use super::util;
use super::Server;

pub fn completion(server: &Server, id: Option<&Value>, msg: &Value) -> Value {
    let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("");
    let line = msg["params"]["position"]["line"].as_u64().unwrap_or(0) as usize;
    let col = msg["params"]["position"]["character"].as_u64().unwrap_or(0) as usize;

    let prefix = util::word_at(&server.documents, uri, line, col);
    let mut items: Vec<Value> = Vec::new();

    if let Some(var_prefix) = prefix.strip_prefix('$') {
        // Variable completion
        if let Some(text) = server.documents.get(uri) {
            for var in util::extract_variables(text) {
                if var_prefix.is_empty() || var.starts_with(var_prefix) {
                    items.push(json!({"label": format!("${var}"), "kind": 6}));
                }
            }
        }
    } else {
        // Command completion
        for name in server.interp.command_names() {
            if prefix.is_empty() || name.starts_with(&prefix) {
                items.push(json!({"label": name, "kind": 3}));
            }
        }
        for name in server.interp.proc_names() {
            if prefix.is_empty() || name.starts_with(&prefix) {
                items.push(json!({"label": name, "kind": 3}));
            }
        }
    }

    json!({"jsonrpc": "2.0", "id": id, "result": {"items": items}})
}

pub fn hover(server: &Server, id: Option<&Value>, msg: &Value) -> Value {
    let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("");
    let line = msg["params"]["position"]["line"].as_u64().unwrap_or(0) as usize;
    let col = msg["params"]["position"]["character"].as_u64().unwrap_or(0) as usize;

    let word = util::word_at(&server.documents, uri, line, col);

    let content = if server.interp.has_command(&word) {
        format!("**{word}** — built-in command")
    } else if let Some(sig) = proc_signature(&server.documents, &word) {
        sig
    } else {
        return json!({"jsonrpc": "2.0", "id": id, "result": null});
    };

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {"contents": {"kind": "markdown", "value": content}}
    })
}

pub fn definition(server: &Server, id: Option<&Value>, msg: &Value) -> Value {
    let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("");
    let line = msg["params"]["position"]["line"].as_u64().unwrap_or(0) as usize;
    let col = msg["params"]["position"]["character"].as_u64().unwrap_or(0) as usize;

    let word = util::word_at(&server.documents, uri, line, col);

    // Search current document first, then others
    for (doc_uri, text) in &server.documents {
        if let Some(def_line) = find_proc_def(text, &word) {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "uri": doc_uri,
                    "range": {
                        "start": {"line": def_line, "character": 0},
                        "end": {"line": def_line, "character": word.len()}
                    }
                }
            });
        }
    }

    json!({"jsonrpc": "2.0", "id": id, "result": null})
}

pub fn references(server: &Server, id: Option<&Value>, msg: &Value) -> Value {
    let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or("");
    let line = msg["params"]["position"]["line"].as_u64().unwrap_or(0) as usize;
    let col = msg["params"]["position"]["character"].as_u64().unwrap_or(0) as usize;

    let word = util::word_at(&server.documents, uri, line, col);
    let mut refs: Vec<Value> = Vec::new();

    for (doc_uri, text) in &server.documents {
        for (i, src_line) in text.lines().enumerate() {
            if util::contains_word(src_line, &word) {
                let ch = src_line.find(&word).unwrap_or(0);
                refs.push(json!({
                    "uri": doc_uri,
                    "range": {
                        "start": {"line": i, "character": ch},
                        "end": {"line": i, "character": ch + word.len()}
                    }
                }));
            }
        }
    }

    json!({"jsonrpc": "2.0", "id": id, "result": refs})
}

fn proc_signature(
    documents: &std::collections::HashMap<String, String>,
    name: &str,
) -> Option<String> {
    for text in documents.values() {
        if let Ok(parsed) = Parser::parse(text) {
            for cmd in &parsed.commands {
                if cmd.words.len() >= 3
                    && cmd.words[0].text() == "proc"
                    && cmd.words[1].text() == name
                {
                    let params = cmd.words[2].text();
                    return Some(format!("```tcl\nproc {name} {{{params}}}\n```"));
                }
            }
        }
    }
    None
}

fn find_proc_def(text: &str, name: &str) -> Option<usize> {
    let parsed = Parser::parse(text).ok()?;
    for cmd in &parsed.commands {
        if cmd.words.len() >= 2 && cmd.words[0].text() == "proc" && cmd.words[1].text() == name {
            return Some(cmd.line.saturating_sub(1));
        }
    }
    None
}
