//! Shared utilities for LSP features.

use std::collections::{HashMap, HashSet};

/// Extract the word under cursor from a document.
pub fn word_at(documents: &HashMap<String, String>, uri: &str, line: usize, col: usize) -> String {
    let Some(text) = documents.get(uri) else {
        return String::new();
    };
    let Some(src_line) = text.lines().nth(line) else {
        return String::new();
    };
    let chars: Vec<char> = src_line.chars().collect();
    let col = col.min(chars.len());
    let start = chars[..col]
        .iter()
        .rposition(|c| is_delimiter(*c))
        .map(|p| p + 1)
        .unwrap_or(0);
    let end = chars[col..]
        .iter()
        .position(|c| is_delimiter(*c))
        .map(|p| col + p)
        .unwrap_or(chars.len());
    chars[start..end].iter().collect()
}

/// Extract variable names from a script ($name patterns).
pub fn extract_variables(text: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut seen = HashSet::new();
    for segment in text.split('$').skip(1) {
        let var: String = segment
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !var.is_empty() && seen.insert(var.clone()) {
            vars.push(var);
        }
    }
    vars
}

/// Check if a line contains a word as a distinct token.
pub fn contains_word(line: &str, word: &str) -> bool {
    line.split(|c: char| is_delimiter(c)).any(|w| w == word)
}

fn is_delimiter(c: char) -> bool {
    c.is_whitespace() || matches!(c, '{' | '}' | '[' | ']' | ';' | '"')
}
