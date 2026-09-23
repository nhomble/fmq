use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::process::Command;

fn read_fixture(dir: &Path, name: &str) -> String {
    fs::read_to_string(dir.join(name))
        .unwrap_or_else(|_| panic!("missing {}", dir.join(name).display()))
        .trim_end()
        .to_string()
}

#[test]
fn queries() {
    let fixtures = Path::new("tests/fixtures/queries");

    for entry in fs::read_dir(fixtures).expect("fixtures/queries not found") {
        let dir = entry.unwrap().path();
        if !dir.is_dir() {
            continue;
        }

        let input = read_fixture(&dir, "input.md") + "\n";
        let expr = read_fixture(&dir, "expr.txt");
        let expected = read_fixture(&dir, "output.txt");

        let result =
            fmq::fmq(&expr, &input, false).unwrap_or_else(|e| panic!("{}: {}", dir.display(), e));

        assert_eq!(
            result.trim_end(),
            expected,
            "failed: {}",
            dir.file_name().unwrap().to_string_lossy()
        );
    }
}

#[test]
fn mutations() {
    let fixtures = Path::new("tests/fixtures/mutations");

    for entry in fs::read_dir(fixtures).expect("fixtures/mutations not found") {
        let dir = entry.unwrap().path();
        if !dir.is_dir() {
            continue;
        }

        let input = read_fixture(&dir, "input.md") + "\n";
        let expr = read_fixture(&dir, "expr.txt");
        let expected = read_fixture(&dir, "output.md");

        let result =
            fmq::fmq(&expr, &input, false).unwrap_or_else(|e| panic!("{}: {}", dir.display(), e));

        assert_eq!(
            result.trim_end(),
            expected,
            "failed: {}",
            dir.file_name().unwrap().to_string_lossy()
        );
    }
}

#[test]
fn errors() {
    let fixtures = Path::new("tests/fixtures/errors");

    for entry in fs::read_dir(fixtures).expect("fixtures/errors not found") {
        let dir = entry.unwrap().path();
        if !dir.is_dir() {
            continue;
        }

        let input = read_fixture(&dir, "input.md") + "\n";
        let expr = read_fixture(&dir, "expr.txt");
        let expected_err = read_fixture(&dir, "error.txt");

        let result = fmq::fmq(&expr, &input, false);

        assert!(
            result.is_err(),
            "{}: expected error, got {:?}",
            dir.display(),
            result
        );

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains(&expected_err),
            "{}: error '{}' should contain '{}'",
            dir.display(),
            err_msg,
            expected_err
        );
    }
}

#[test]
fn init() {
    let fixtures = Path::new("tests/fixtures/init");

    for entry in fs::read_dir(fixtures).expect("fixtures/init not found") {
        let dir = entry.unwrap().path();
        if !dir.is_dir() {
            continue;
        }

        let input = read_fixture(&dir, "input.md") + "\n";
        let expr = read_fixture(&dir, "expr.txt");

        let is_mutation = dir.join("output.md").exists();
        let expected = if is_mutation {
            read_fixture(&dir, "output.md")
        } else {
            read_fixture(&dir, "output.txt")
        };

        let reader = Cursor::new(input);
        let result = fmq::fmq_reader(&expr, reader, true)
            .unwrap_or_else(|e| panic!("{}: {}", dir.display(), e));

        assert_eq!(
            result.trim_end(),
            expected,
            "failed: {}",
            dir.file_name().unwrap().to_string_lossy()
        );
    }
}

#[test]
fn in_place() {
    let fixtures = Path::new("tests/fixtures/mutations");

    for entry in fs::read_dir(fixtures).expect("fixtures/mutations not found") {
        let dir = entry.unwrap().path();
        if !dir.is_dir() {
            continue;
        }

        let input = read_fixture(&dir, "input.md") + "\n";
        let expr = read_fixture(&dir, "expr.txt");
        let expected = read_fixture(&dir, "output.md");

        // Create temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!(
            "fmq-test-{}.md",
            dir.file_name().unwrap().to_string_lossy()
        ));
        fs::write(&temp_file, &input).unwrap();

        // Run CLI with --in-place
        let output = Command::new(env!("CARGO_BIN_EXE_fmq"))
            .arg(&expr)
            .arg(&temp_file)
            .arg("--in-place")
            .output()
            .expect("failed to execute fmq");

        assert!(
            output.status.success(),
            "fmq failed for {}: {}",
            dir.display(),
            String::from_utf8_lossy(&output.stderr)
        );

        // Verify file was modified
        let result = fs::read_to_string(&temp_file).unwrap();
        assert_eq!(
            result.trim_end(),
            expected,
            "failed: {}",
            dir.file_name().unwrap().to_string_lossy()
        );

        // Cleanup
        fs::remove_file(&temp_file).ok();
    }
}

#[test]
fn in_place_rejects_non_object_result() {
    // --in-place requires the expression result to be an object
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("fmq-test-non-object.md");

    let original = "---\ntitle: Hello\n---\nBody\n";
    fs::write(&temp_file, original).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_fmq"))
        .arg(".title != \"x\"")
        .arg(&temp_file)
        .arg("--in-place")
        .output()
        .expect("failed to execute fmq");

    assert!(
        !output.status.success(),
        "should fail for non-object result"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("must be an object"),
        "error should mention 'must be an object': {}",
        stderr
    );

    let result = fs::read_to_string(&temp_file).unwrap();
    assert_eq!(result, original, "file should be unchanged");

    fs::remove_file(&temp_file).ok();
}

#[test]
fn cli_prints_all_query_outputs() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("fmq-test-multi-output.md");

    let original = "---\ntags:\n  - a\n  - b\n  - c\n---\nBody.\n";
    fs::write(&temp_file, original).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_fmq"))
        .arg(".tags[]")
        .arg(&temp_file)
        .output()
        .expect("failed to execute fmq");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"a\nb\nc\n");

    let output = Command::new(env!("CARGO_BIN_EXE_fmq"))
        .arg("empty")
        .arg(&temp_file)
        .output()
        .expect("failed to execute fmq");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"");

    fs::remove_file(&temp_file).ok();
}

#[test]
fn in_place_rejects_empty_output() {
    // --in-place requires exactly one output; `empty` produces none.
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("fmq-test-empty-output.md");

    let original = "---\ntitle: Hello\n---\nBody\n";
    fs::write(&temp_file, original).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_fmq"))
        .arg("empty")
        .arg(&temp_file)
        .arg("--in-place")
        .output()
        .expect("failed to execute fmq");

    assert!(!output.status.success(), "should fail for empty result");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no output"),
        "error should mention 'no output': {}",
        stderr
    );

    let result = fs::read_to_string(&temp_file).unwrap();
    assert_eq!(result, original, "file should be unchanged");

    fs::remove_file(&temp_file).ok();
}

#[test]
fn in_place_leaves_no_temp_files() {
    let dir = std::env::temp_dir().join("fmq-atomic-leaves-no-temp-files");
    if dir.exists() {
        fs::remove_dir_all(&dir).ok();
    }
    fs::create_dir_all(&dir).unwrap();

    let doc = dir.join("doc.md");
    fs::write(&doc, "---\ntitle: Hello\n---\nBody\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_fmq"))
        .arg(".title = \"Bye\"")
        .arg(&doc)
        .arg("--in-place")
        .output()
        .expect("failed to execute fmq");

    assert!(output.status.success(), "expected success: {:?}", output);

    let result = fs::read_to_string(&doc).unwrap();
    assert!(result.contains("title: Bye"), "result: {}", result);
    assert!(result.contains("Body"), "result: {}", result);

    let entries = fs::read_dir(&dir).unwrap().count();
    assert_eq!(entries, 1, "expected no leftover temp files in {:?}", dir);

    fs::remove_dir_all(&dir).ok();
}

#[cfg(unix)]
#[test]
fn in_place_preserves_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let dir = std::env::temp_dir().join("fmq-atomic-preserves-permissions");
    if dir.exists() {
        fs::remove_dir_all(&dir).ok();
    }
    fs::create_dir_all(&dir).unwrap();

    let doc = dir.join("doc.md");
    fs::write(&doc, "---\ntitle: Hello\n---\nBody\n").unwrap();
    fs::set_permissions(&doc, fs::Permissions::from_mode(0o640)).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_fmq"))
        .arg(".title = \"Bye\"")
        .arg(&doc)
        .arg("--in-place")
        .output()
        .expect("failed to execute fmq");

    assert!(output.status.success(), "expected success: {:?}", output);

    let mode = fs::metadata(&doc).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o640, "permissions should be preserved");

    fs::remove_dir_all(&dir).ok();
}

#[cfg(unix)]
#[test]
fn in_place_through_symlink_updates_target() {
    use std::os::unix::fs::symlink;

    let dir = std::env::temp_dir().join("fmq-atomic-symlink");
    if dir.exists() {
        fs::remove_dir_all(&dir).ok();
    }
    fs::create_dir_all(&dir).unwrap();

    let real = dir.join("real.md");
    let link = dir.join("link.md");
    fs::write(&real, "---\ntitle: Hello\n---\nBody\n").unwrap();
    symlink(&real, &link).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_fmq"))
        .arg(".title = \"Bye\"")
        .arg(&link)
        .arg("--in-place")
        .output()
        .expect("failed to execute fmq");

    assert!(output.status.success(), "expected success: {:?}", output);

    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink(),
        "link.md should still be a symlink"
    );

    let result = fs::read_to_string(&real).unwrap();
    assert!(result.contains("title: Bye"), "result: {}", result);

    fs::remove_dir_all(&dir).ok();
}

#[cfg(unix)]
#[test]
fn in_place_failure_leaves_original_intact() {
    use std::os::unix::fs::PermissionsExt;

    let dir = std::env::temp_dir().join("fmq-atomic-failure-intact");
    if dir.exists() {
        fs::remove_dir_all(&dir).ok();
    }
    fs::create_dir_all(&dir).unwrap();

    let doc = dir.join("doc.md");
    let original = "---\ntitle: Hello\n---\nBody\n";
    fs::write(&doc, original).unwrap();

    fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).unwrap();

    if fs::write(dir.join("probe"), "").is_ok() {
        // Running with elevated privileges (e.g. root in CI); permissions
        // are not actually enforced, so this test cannot validate anything.
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        fs::remove_dir_all(&dir).ok();
        return;
    }

    let output = Command::new(env!("CARGO_BIN_EXE_fmq"))
        .arg(".title = \"Bye\"")
        .arg(&doc)
        .arg("--in-place")
        .output()
        .expect("failed to execute fmq");

    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();

    assert!(!output.status.success(), "expected failure: {:?}", output);

    let result = fs::read_to_string(&doc).unwrap();
    assert_eq!(result, original, "file should be unchanged");

    fs::remove_dir_all(&dir).ok();
}
