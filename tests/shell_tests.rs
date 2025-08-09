// tests/shell_tests.rs

use croner::shell::{get_command, get_command_as_os_str};
use std::ffi::OsString;

#[test]
fn empty_command_returns_empty_vec() {
    assert_eq!(get_command(""), Vec::<String>::new());
}

#[test]
fn single_token_fast_path() {
    assert_eq!(get_command("ls"), vec!["ls"]);
}

#[test]
fn whitespace_split_fast_path() {
    assert_eq!(get_command("ls -la /tmp"), vec!["ls", "-la", "/tmp"]);
}

#[test]
fn quoted_strings_single_quotes() {
    assert_eq!(
        get_command("echo 'hello world'"),
        vec!["echo", "hello world"]
    );
}

#[test]
fn quoted_strings_double_quotes() {
    assert_eq!(
        get_command("echo \"hello world\""),
        vec!["echo", "hello world"]
    );
}

#[test]
fn escaped_characters_in_double_quotes() {
    assert_eq!(
        get_command("echo \"line1\\nline2\""),
        vec!["echo", "line1\nline2"]
    );
}

#[test]
fn escaped_characters_outside_quotes() {
    assert_eq!(
        get_command("echo line1\\nline2"),
        vec!["echo", "line1\nline2"]
    );
}

#[test]
fn mixed_quoting_and_args() {
    assert_eq!(
        get_command("cmd 'arg with spaces' plain \"another one\""),
        vec!["cmd", "arg with spaces", "plain", "another one"]
    );
}

#[test]
fn multiple_spaces_are_ignored_between_tokens() {
    assert_eq!(get_command("ls     -la   /tmp"), vec!["ls", "-la", "/tmp"]);
}

#[test]
fn utf8_token_is_parsed_correctly() {
    assert_eq!(get_command("echo café"), vec!["echo", "café"]);
}

#[test]
fn dangling_backslash_returns_error() {
    assert_eq!(get_command("echo \\"), Vec::<String>::new());
}

#[test]
fn unclosed_single_quote_returns_error() {
    assert_eq!(get_command("echo 'oops"), Vec::<String>::new());
}

#[test]
fn unclosed_double_quote_returns_error() {
    assert_eq!(get_command("echo \"oops"), Vec::<String>::new());
}

#[test]
fn get_command_as_os_str_matches_get_command() {
    let s = "echo hello world";
    let expected: Vec<OsString> = get_command(s).into_iter().map(OsString::from).collect();
    assert_eq!(get_command_as_os_str(s), expected);
}

//
// Platform-specific invalid UTF-8 tests
//
#[cfg(unix)]
#[test]
fn invalid_utf8_token_returns_error_unix() {
    use std::os::unix::ffi::OsStringExt;

    // Raw bytes that are invalid UTF-8
    let bytes = vec![0xff, 0xfe, b'a'];
    let _os_str = OsString::from_vec(bytes.clone());

    let input = String::from_utf8_lossy(&bytes).to_string();
    let parsed = get_command(&format!("echo {}", input));

    assert_eq!(parsed.first().map(String::as_str), Some("echo"));
}

#[cfg(windows)]
#[test]
fn invalid_utf8_token_returns_error_windows() {
    use std::os::windows::ffi::OsStringExt;

    // UTF-16 values that won't map cleanly to UTF-8 (simulate invalid UTF-8 path)
    let wide: [u16; 3] = [0xD800, 0xDC00, b'a' as u16]; // Surrogate pair + ASCII 'a'
    let _os_str = OsString::from_wide(&wide);

    let input = String::from_utf16_lossy(&wide);
    let parsed = get_command(&format!("echo {}", input));

    assert_eq!(parsed.first().map(String::as_str), Some("echo"));
}
