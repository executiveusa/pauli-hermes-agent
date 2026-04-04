#[cfg(test)]
mod tests {
    use pi::extensions::{DangerousCommandClass, classify_dangerous_command};

    #[test]
    fn repro_rm_rf_dot_slash_bypass() {
        let cmd = "rm";
        let args = vec!["-rf".to_string(), "/.".to_string()];
        let classes = classify_dangerous_command(cmd, &args);

        // Should be detected as RecursiveDelete
        assert!(
            classes.contains(&DangerousCommandClass::RecursiveDelete),
            "rm -rf /. should be classified as RecursiveDelete, got {classes:?}",
        );
    }
}
