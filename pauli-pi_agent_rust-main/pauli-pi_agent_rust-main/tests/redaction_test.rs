use pi::extensions::{SecretBrokerPolicy, redact_command_for_logging};

#[test]
fn test_redaction_escaped_quotes() {
    let policy = SecretBrokerPolicy {
        enabled: true,
        secret_suffixes: vec!["_KEY".to_string()],
        secret_prefixes: vec![],
        secret_exact: vec![],
        disclosure_allowlist: vec![],
        redaction_placeholder: "[REDACTED]".to_string(),
    };

    // Case 1: Simple quoted value
    let cmd = r#"export API_KEY="secret""#;
    let redacted = redact_command_for_logging(&policy, cmd);
    assert_eq!(redacted, r"export API_KEY=[REDACTED]");

    // Case 2: Escaped double quote inside double quotes
    // If the regex is broken, it stops at the first escaped quote.
    let cmd = r#"export API_KEY="secret "key"""#;
    let redacted = redact_command_for_logging(&policy, cmd);

    // We expect the entire string to be redacted.
    assert_eq!(redacted, r"export API_KEY=[REDACTED]");
}
