use pi::tools::{ReadTool, Tool};
use serde_json::json;
use std::io::Write;

#[test]
fn test_read_large_file_offset() {
    futures::executor::block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("large.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();

        // Write ~300KB of data (approx 60,000 lines like "lineX").
        for i in 1..=60000 {
            writeln!(file, "line{i}").unwrap();
        }

        let tool = ReadTool::new(dir.path());
        let offset = 50000;
        let limit = 10;

        let result = tool
            .execute(
                "test",
                json!({
                    "path": "large.txt",
                    "offset": offset,
                    "limit": limit
                }),
                None,
            )
            .await
            .unwrap();

        let content = match &result.content[0] {
            pi::model::ContentBlock::Text(t) => t.text.clone(),
            _ => panic!("Expected text content"),
        };

        println!("Content sample: {content:.100}...");

        assert!(
            !content.contains("Offset 50000 is beyond end of file"),
            "Bug reproduced: Offset considered beyond EOF due to early truncation"
        );

        assert!(content.contains("line50000"), "Should contain line50000");
    });
}
