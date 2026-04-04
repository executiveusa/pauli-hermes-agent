#[cfg(test)]
mod tests {
    use pi::tools::{EditTool, Tool};
    use serde_json::json;

    #[test]
    fn test_edit_empty_old_text_is_rejected() {
        asupersync::test_utils::run_test(|| async {
            let tmp = tempfile::tempdir().unwrap();
            let file_path = tmp.path().join("test.txt");
            std::fs::write(&file_path, "content").unwrap();

            let tool = EditTool::new(tmp.path());
            let err = tool
                .execute(
                    "test",
                    json!({
                        "path": file_path.to_string_lossy(),
                        "oldText": "",
                        "newText": "PREFIX"
                    }),
                    None,
                )
                .await
                .expect_err("empty oldText should be rejected");

            let msg = err.to_string();
            assert!(
                msg.contains("old text cannot be empty"),
                "unexpected error: {msg}"
            );
            let content = std::fs::read_to_string(&file_path).unwrap();
            assert_eq!(content, "content", "file should remain unchanged");
        });
    }
}
