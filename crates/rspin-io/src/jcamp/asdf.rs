use rspin_core::{RSpinError, Result};

#[derive(Clone, Copy)]
enum AsdfCode {
    Absolute(NumberPrefix),
    Difference(NumberPrefix),
    Duplicate(char),
}

#[derive(Clone, Copy)]
enum NumberPrefix {
    Positive(char),
    Negative(char),
}

#[derive(Clone, Copy)]
enum LastEmission {
    Absolute(f64),
    Difference(f64),
}

pub(super) fn decode_values(field: &'static str, line: &str) -> Result<Vec<f64>> {
    let chars = line.chars().collect::<Vec<_>>();
    let mut values = Vec::new();
    let mut index = 0;
    let mut last_value = None;
    let mut last_emission = None;

    while index < chars.len() {
        if is_separator(chars[index]) {
            index += 1;
            continue;
        }

        if let Some(code) = asdf_code(chars[index]) {
            let (suffix, next_index) = read_asdf_suffix(&chars, index + 1);
            apply_asdf_code(
                field,
                code,
                &suffix,
                &mut values,
                &mut last_value,
                &mut last_emission,
            )?;
            index = next_index;
            continue;
        }

        if is_affn_start(chars[index]) {
            let (token, next_index) = read_affn_token(&chars, index);
            let value = super::parse_float(field, &token)?;
            values.push(value);
            last_value = Some(value);
            last_emission = Some(LastEmission::Absolute(value));
            index = next_index;
            continue;
        }

        return Err(parse_error(field, "unsupported ASDF token"));
    }

    Ok(values)
}

fn apply_asdf_code(
    field: &'static str,
    code: AsdfCode,
    suffix: &str,
    values: &mut Vec<f64>,
    last_value: &mut Option<f64>,
    last_emission: &mut Option<LastEmission>,
) -> Result<()> {
    match code {
        AsdfCode::Absolute(prefix) => {
            let value = parse_prefixed_number(field, prefix, suffix)?;
            values.push(value);
            *last_value = Some(value);
            *last_emission = Some(LastEmission::Absolute(value));
        }
        AsdfCode::Difference(prefix) => {
            let previous = require_last_value(field, *last_value)?;
            let delta = parse_prefixed_number(field, prefix, suffix)?;
            let value = add_difference(field, previous, delta)?;
            values.push(value);
            *last_value = Some(value);
            *last_emission = Some(LastEmission::Difference(delta));
        }
        AsdfCode::Duplicate(prefix) => {
            repeat_last_emission(field, prefix, suffix, values, last_value, *last_emission)?;
        }
    }
    Ok(())
}

fn repeat_last_emission(
    field: &'static str,
    prefix: char,
    suffix: &str,
    values: &mut Vec<f64>,
    last_value: &mut Option<f64>,
    last_emission: Option<LastEmission>,
) -> Result<()> {
    let count = parse_duplicate_count(field, prefix, suffix)?;
    let Some(emission) = last_emission else {
        return Err(parse_error(field, "DUP code without a previous value"));
    };

    for _ in 1..count {
        match emission {
            LastEmission::Absolute(value) => {
                values.push(value);
                *last_value = Some(value);
            }
            LastEmission::Difference(delta) => {
                let previous = require_last_value(field, *last_value)?;
                let value = add_difference(field, previous, delta)?;
                values.push(value);
                *last_value = Some(value);
            }
        }
    }
    Ok(())
}

fn parse_prefixed_number(field: &'static str, prefix: NumberPrefix, suffix: &str) -> Result<f64> {
    let mut text = String::new();
    match prefix {
        NumberPrefix::Positive(digit) => text.push(digit),
        NumberPrefix::Negative(digit) => {
            text.push('-');
            text.push(digit);
        }
    }
    text.push_str(suffix);
    super::parse_float(field, &text)
}

fn parse_duplicate_count(field: &'static str, prefix: char, suffix: &str) -> Result<usize> {
    let mut text = String::from(prefix);
    text.push_str(suffix);
    text.parse::<usize>().map_err(|error| RSpinError::Parse {
        format: "JCAMP-DX",
        message: format!("{field}: {error}"),
    })
}

fn require_last_value(field: &'static str, value: Option<f64>) -> Result<f64> {
    if let Some(value) = value {
        Ok(value)
    } else {
        Err(parse_error(field, "DIF code without a previous value"))
    }
}

fn add_difference(field: &'static str, previous: f64, delta: f64) -> Result<f64> {
    let value = previous + delta;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(RSpinError::NonFinite { field })
    }
}

fn read_asdf_suffix(chars: &[char], mut index: usize) -> (String, usize) {
    let mut suffix = String::new();
    while index < chars.len() && (chars[index].is_ascii_digit() || chars[index] == '.') {
        suffix.push(chars[index]);
        index += 1;
    }
    (suffix, index)
}

fn read_affn_token(chars: &[char], start: usize) -> (String, usize) {
    let mut token = String::new();
    let mut index = start;
    let mut has_exponent = false;

    while index < chars.len() {
        let character = chars[index];
        if is_separator(character) {
            break;
        }

        if is_exponent_start(
            character,
            &token,
            has_exponent,
            chars.get(index + 1).copied(),
        ) {
            has_exponent = true;
            token.push(character);
            index += 1;
            continue;
        }

        if index > start && asdf_code(character).is_some() {
            break;
        }

        if (character == '+' || character == '-')
            && index > start
            && !token_ends_in_exponent(&token)
        {
            break;
        }

        if is_affn_body(character) {
            token.push(character);
            index += 1;
            continue;
        }

        break;
    }

    (token, index)
}

fn asdf_code(character: char) -> Option<AsdfCode> {
    match character {
        '@' => Some(AsdfCode::Absolute(NumberPrefix::Positive('0'))),
        '%' => Some(AsdfCode::Difference(NumberPrefix::Positive('0'))),
        _ => positive_digit("ABCDEFGHI", character)
            .map(|digit| AsdfCode::Absolute(NumberPrefix::Positive(digit)))
            .or_else(|| {
                positive_digit("abcdefghi", character)
                    .map(|digit| AsdfCode::Absolute(NumberPrefix::Negative(digit)))
            })
            .or_else(|| {
                positive_digit("JKLMNOPQR", character)
                    .map(|digit| AsdfCode::Difference(NumberPrefix::Positive(digit)))
            })
            .or_else(|| {
                positive_digit("jklmnopqr", character)
                    .map(|digit| AsdfCode::Difference(NumberPrefix::Negative(digit)))
            })
            .or_else(|| positive_digit("STUVWXYZs", character).map(AsdfCode::Duplicate)),
    }
}

fn positive_digit(table: &str, character: char) -> Option<char> {
    table
        .chars()
        .zip(['1', '2', '3', '4', '5', '6', '7', '8', '9'])
        .find_map(|(candidate, digit)| {
            if candidate == character {
                Some(digit)
            } else {
                None
            }
        })
}

fn is_separator(character: char) -> bool {
    character.is_ascii_whitespace() || character == ',' || character == ';'
}

fn is_affn_start(character: char) -> bool {
    character.is_ascii_digit() || character == '.' || character == '+' || character == '-'
}

fn is_affn_body(character: char) -> bool {
    character.is_ascii_digit()
        || character == '.'
        || character == '+'
        || character == '-'
        || character == 'E'
        || character == 'e'
}

fn is_exponent_start(character: char, token: &str, has_exponent: bool, next: Option<char>) -> bool {
    !has_exponent
        && (character == 'E' || character == 'e')
        && token
            .chars()
            .any(|token_character| token_character.is_ascii_digit())
        && matches!(next, Some('+' | '-'))
}

fn token_ends_in_exponent(token: &str) -> bool {
    let mut chars = token.chars().rev();
    if let Some(character) = chars.next() {
        character == 'E' || character == 'e'
    } else {
        false
    }
}

fn parse_error(field: &'static str, message: &str) -> RSpinError {
    RSpinError::Parse {
        format: "JCAMP-DX",
        message: format!("{field}: {message}"),
    }
}
