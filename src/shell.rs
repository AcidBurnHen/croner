use std::ffi::OsString;

#[inline]
pub fn get_command(input: &str) -> Vec<String> {
    parse_command(input).unwrap_or_default()
}

#[inline]
pub fn get_command_as_os_str(input: &str) -> Vec<OsString> {
    parse_command(input)
        .unwrap_or_default()
        .into_iter()
        .map(OsString::from)
        .collect()
}

#[inline]
fn parse_command(input: &str) -> Result<Vec<String>, &'static str> {
    let input_bytes = input.as_bytes();

    // Empty command
    if input_bytes.is_empty() {
        return Ok(Vec::new());
    }

    // Simple multi token command
    if is_single_token(input_bytes) {
        return Ok(vec![input.to_owned()]);
    }

    // Fast path B: no quotes/backslashes (but may have whitespace) → split_whitespace
    if has_no_quotes_or_backslashes(input_bytes) {
        return Ok(input.split_whitespace().map(|s| s.to_owned()).collect());
    }

    parse_command_full(input_bytes)
}

// Checks if the command can be parsed fast or if it contains parts that require the full parser
#[inline]
fn is_single_token(bytes: &[u8]) -> bool {
    !bytes
        .iter()
        .any(|&b| matches!(b, b'\'' | b'"' | b'\\') || is_ascii_whitespace(b))
}

#[inline]
fn has_no_quotes_or_backslashes(bytes: &[u8]) -> bool {
    !bytes.iter().any(|&b| matches!(b, b'\'' | b'"' | b'\\'))
}

#[inline]
fn is_ascii_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C)
}

#[inline]
fn estimate_tokens(command_bytes: &[u8]) -> usize {
    // Counts how many separate tokens are in the input
    // by counting transitions from whitespace -> non-whitespace.
    let total_len = command_bytes.len();
    let mut token_count = 0usize;
    let mut inside_whitespace = true;

    let mut index = 0usize;
    while index < total_len {
        let byte = command_bytes[index];
        let is_whitespace = is_ascii_whitespace(byte);
        if inside_whitespace && !is_whitespace {
            token_count += 1;
        }
        inside_whitespace = is_whitespace;
        index += 1;
    }

    token_count.max(1)
}

#[inline(always)]
fn push_token(args: &mut Vec<String>, token_buf: &mut Vec<u8>) -> Result<(), &'static str> {
    if token_buf.is_empty() {
        return Ok(());
    }
    match String::from_utf8(std::mem::take(token_buf)) {
        Ok(token) => {
            args.push(token);
            Ok(())
        }
        Err(_) => Err("invalid utf-8 in token"),
    }
}

fn parse_command_full(input_bytes: &[u8]) -> Result<Vec<String>, &'static str> {
    let mut args: Vec<String> = Vec::with_capacity(estimate_tokens(input_bytes));
    let mut token_buffer: Vec<u8> = Vec::with_capacity(input_bytes.len());

    let mut index = 0usize;
    let input_len = input_bytes.len();

    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut escaping = false;

    while index < input_len {
        let byte = input_bytes[index];

        if escaping {
            match byte {
                b'n' => token_buffer.push(b'\n'),
                b't' => token_buffer.push(b'\t'),
                b'r' => token_buffer.push(b'\r'),
                _ => token_buffer.push(byte),
            }
            escaping = false;
            index += 1;
            continue;
        }

        if in_single_quotes {
            if byte == b'\'' {
                in_single_quotes = false;
            } else {
                token_buffer.push(byte)
            }
            index += 1;
            continue;
        }

        if in_double_quotes {
            if byte == b'"' {
                in_double_quotes = false;
            } else if byte == b'\\' {
                escaping = true;
            } else {
                token_buffer.push(byte);
            }
            index += 1;
            continue;
        }

        match byte {
            b'\\' => {
                escaping = true;
            }
            b'\'' => {
                in_single_quotes = true;
            }
            b'"' => {
                in_double_quotes = true;
            }
            b if is_ascii_whitespace(b) => {
                push_token(&mut args, &mut token_buffer)?;
                index += 1;
                while index < input_len && is_ascii_whitespace(input_bytes[index]) {
                    index += 1;
                }
                continue;
            }
            _ => token_buffer.push(byte),
        }

        index += 1;
    }

    if escaping {
        return Err("dangling backslash at end of input");
    }
    if in_single_quotes || in_double_quotes {
        return Err("unclosed quote");
    }

    if !token_buffer.is_empty() {
        match String::from_utf8(token_buffer) {
            Ok(token) => args.push(token),
            Err(_) => return Err("invalid utf-8 in token"),
        }
    }

    Ok(args)
}
