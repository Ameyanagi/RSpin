//! Agilent/Varian `procpar` parsing helpers.

use std::collections::BTreeMap;

use rspin_core::{RSpinError, Result};

pub(super) fn parse_procpar(input: &str) -> BTreeMap<String, Vec<String>> {
    let mut parameters = BTreeMap::new();
    let mut lines = input.lines();
    while let Some(definition) = lines.next() {
        let mut parts = definition.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        let Some(values) = read_counted_values(&mut lines) else {
            break;
        };
        parameters.insert(normalized_key(name), values);
        let _ = read_counted_values(&mut lines);
    }
    parameters
}

pub(super) fn first_text(parameters: &BTreeMap<String, Vec<String>>, key: &str) -> Option<String> {
    parameters
        .get(key)
        .and_then(|values| values.first())
        .cloned()
}

pub(super) fn first_f64(
    parameters: &BTreeMap<String, Vec<String>>,
    key: &'static str,
) -> Result<Option<f64>> {
    match parameters.get(key).and_then(|values| values.first()) {
        Some(value) => {
            let parsed = value.parse::<f64>().map_err(|error| RSpinError::Parse {
                format: "Agilent",
                message: format!("{key}: {error}"),
            })?;
            if !parsed.is_finite() {
                return Err(RSpinError::NonFinite { field: key });
            }
            Ok(Some(parsed))
        }
        None => Ok(None),
    }
}

pub(super) fn first_usize(
    parameters: &BTreeMap<String, Vec<String>>,
    key: &'static str,
) -> Result<Option<usize>> {
    match parameters.get(key).and_then(|values| values.first()) {
        Some(value) => {
            let parsed = value.parse::<usize>().map_err(|error| RSpinError::Parse {
                format: "Agilent",
                message: format!("{key}: {error}"),
            })?;
            Ok(Some(parsed))
        }
        None => Ok(None),
    }
}

fn read_counted_values(lines: &mut std::str::Lines<'_>) -> Option<Vec<String>> {
    let first_line = lines.next()?;
    let mut first_tokens = tokens(first_line);
    let expected = first_tokens.first()?.parse::<usize>().ok()?;
    let _ = first_tokens.remove(0);

    let mut values = first_tokens;
    while values.len() < expected {
        let Some(line) = lines.next() else {
            break;
        };
        values.extend(tokens(line));
    }
    values.truncate(expected);
    values.len().eq(&expected).then_some(values)
}

fn tokens(line: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for character in line.chars() {
        match character {
            '"' => {
                if in_quotes {
                    values.push(current.clone());
                    current.clear();
                    in_quotes = false;
                } else {
                    push_token(&mut values, &mut current);
                    in_quotes = true;
                }
            }
            value if value.is_whitespace() && !in_quotes => push_token(&mut values, &mut current),
            value => current.push(value),
        }
    }
    push_token(&mut values, &mut current);
    values
}

fn push_token(values: &mut Vec<String>, current: &mut String) {
    let token = current.trim();
    if !token.is_empty() {
        values.push(token.to_owned());
    }
    current.clear();
}

fn normalized_key(key: &str) -> String {
    key.chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}
