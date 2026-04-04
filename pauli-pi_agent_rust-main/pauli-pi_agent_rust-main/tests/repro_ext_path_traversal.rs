use pi::extensions_js::PiJsRuntime;
use std::fs;
use tempfile::TempDir;

#[test]
fn repro_ext_path_traversal() {
    futures::executor::block_on(async {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let ext_root = root.join("ext");
        fs::create_dir(&ext_root).unwrap();

        let secret_file = root.join("secret.js");
        fs::write(&secret_file, "export const secret = 's3cr3t';").unwrap();

        // index.js inside ext_root tries to import ../secret.js
        let index_file = ext_root.join("index.js");
        fs::write(
            &index_file,
            "import { secret } from '../secret.js'; globalThis.secret = secret;",
        )
        .unwrap();

        let runtime = PiJsRuntime::new().await.unwrap();

        // Register extension root
        runtime.add_extension_root(ext_root.clone());

        // Use dynamic import because `eval()`/`eval_file()` run as scripts, not modules.
        let script = format!(
            r"
            globalThis.traversalAttempt = {{}};
            import({index_file:?}).then(() => {{
                globalThis.traversalAttempt.ok = true;
            }}).catch((err) => {{
                globalThis.traversalAttempt.ok = false;
                globalThis.traversalAttempt.error = String((err && err.message) || err || '');
            }}).finally(() => {{
                globalThis.traversalAttempt.done = true;
            }});
            "
        );
        runtime.eval(&script).await.unwrap();

        let result = runtime.read_global_json("traversalAttempt").await.unwrap();
        assert_eq!(result["done"], serde_json::json!(true));
        assert_eq!(result["ok"], serde_json::json!(false));
        let error = result["error"].as_str().unwrap_or_default();
        println!("Got expected error: {error}");
        assert!(
            error.contains("Module path escapes extension root") && error.contains("secret.js"),
            "Unexpected error message: {error}",
        );
    });
}
