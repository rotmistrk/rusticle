//! Publish diagnostics from rusticle's validator.

use crate::validate::Severity;
use serde_json::{json, Value};

use super::Server;

pub fn publish(server: &Server, uri: &str) -> Value {
    let diagnostics = if let Some(text) = server.documents.get(uri) {
        let result = server.interp.validate(text);
        result
            .errors
            .iter()
            .chain(result.warnings.iter())
            .map(|d| {
                let line = d.location.line.saturating_sub(1);
                let severity = match d.severity {
                    Severity::Error => 1,
                    Severity::Warning => 2,
                };
                json!({
                    "range": {
                        "start": {"line": line, "character": 0},
                        "end": {"line": line, "character": 100}
                    },
                    "severity": severity,
                    "source": "rusticle",
                    "message": d.message
                })
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {"uri": uri, "diagnostics": diagnostics}
    })
}
