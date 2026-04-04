#[cfg(test)]
mod tests {
    use pi::tools::{EditTool, Tool};
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_edit_trailing_whitespace_fuzzy() {
        asupersync::test_utils::run_test(|| async {
            let tmp = tempdir().unwrap();
            let file_path = tmp.path().join("fuzzy.txt");

            // File has "foo " (with trailing space)
            fs::write(&file_path, "foo ").unwrap();

            let tool = EditTool::new(tmp.path());

            // Case 1: User tries to replace "foo" (no space) with "bar" (no space).
            // Expectation: Fuzzy match works, but ignores/skips trailing space in file.
            // Result: "bar " (space preserved).
            let output = tool
                .execute(
                    "call1",
                    json!({
                        "path": "fuzzy.txt",
                        "oldText": "foo",
                        "newText": "bar"
                    }),
                    None,
                )
                .await
                .unwrap();

            assert!(!output.is_error);
            let content = fs::read_to_string(&file_path).unwrap();
            assert_eq!(content, "bar ");

            // Case 2: User tries to replace "bar " (with space) with "baz" (no space).
            // Expectation: Exact match works.
            // Result: "baz" (space deleted).
            let output = tool
                .execute(
                    "call2",
                    json!({
                        "path": "fuzzy.txt",
                        "oldText": "bar ",
                        "newText": "baz"
                    }),
                    None,
                )
                .await
                .unwrap();

            assert!(!output.is_error);
            let content = fs::read_to_string(&file_path).unwrap();
            assert_eq!(content, "baz");

            // Case 3: User tries to replace "baz" (no space) with "qux" (no space).
            // But let's say the file has "baz  " (2 spaces).
            fs::write(&file_path, "baz  ").unwrap();

            // User provides "baz " (1 space). This is an exact substring match at the
            // beginning of "baz  ", so edit replacement happens in exact mode first.
            // Result: "qux " (one trailing space remains).

            let output = tool
                .execute(
                    "call3",
                    json!({
                        "path": "fuzzy.txt",
                        "oldText": "baz ",
                        "newText": "qux"
                    }),
                    None,
                )
                .await
                .unwrap();

            assert!(!output.is_error);
            let content = fs::read_to_string(&file_path).unwrap();
            assert_eq!(content, "qux ");
        });
    }
}
