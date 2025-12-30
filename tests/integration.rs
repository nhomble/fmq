use std::fs;
use std::path::Path;

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

        let result = fmq::fmq(&expr, &input).unwrap_or_else(|e| panic!("{}: {}", dir.display(), e));

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

        let result = fmq::fmq(&expr, &input).unwrap_or_else(|e| panic!("{}: {}", dir.display(), e));

        assert_eq!(
            result.trim_end(),
            expected,
            "failed: {}",
            dir.file_name().unwrap().to_string_lossy()
        );
    }
}
