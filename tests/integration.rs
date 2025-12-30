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
fn in_place_rejects_non_mutation() {
    // --in-place requires a mutation expression
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("fmq-test-non-mutation.md");

    fs::write(&temp_file, "---\ntitle: Hello\n---\nBody\n").unwrap();

    // Expression uses + not +=, so is_mutation returns false
    let output = Command::new(env!("CARGO_BIN_EXE_fmq"))
        .arg(". + {topics: [\"ai\"]}")
        .arg(&temp_file)
        .arg("--in-place")
        .output()
        .expect("failed to execute fmq");

    assert!(
        !output.status.success(),
        "should fail for non-mutation expression"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("mutation"),
        "error should mention mutation: {}",
        stderr
    );

    fs::remove_file(&temp_file).ok();
}
