use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::process::Command;

use lazy_static;
use tempfile::TempDir;

lazy_static::lazy_static! {
    static ref BASE_DIR: PathBuf = Path::new("tests").join("simpla_programs");
}

#[test]
fn test_source_bytecode_translation() {
    run_test(bytecode_translation_test);
}

fn bytecode_translation_test(file: &Path, target: &tempfile::TempDir) {
    let target_name = format!("{}c", file.file_name().unwrap().to_str().unwrap());
    let simpla_bytecode = target.path().join(target_name);
    run_compile("translate", file, &simpla_bytecode);
}

fn run_test<F>(callback: F)
where
    F: Fn(&Path, &tempfile::TempDir),
{
    let target_dir = TempDir::new().unwrap();
    for entry in read_dir(BASE_DIR.as_path()).unwrap() {
        let file = entry.unwrap().path();
        callback(&file, &target_dir)
    }
}

fn run_compile(action: &str, src: &Path, dst: &Path) {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg(action)
        .arg(src)
        .arg(dst)
        .output();
    let output = output.unwrap();
    assert!(
        output.status.success(),
        "STDOUT:\n{}\nSTDERR:\n{}\n",
        string(&output.stdout),
        string(&output.stderr)
    );
}

fn string(v: &[u8]) -> &str {
    std::str::from_utf8(v).unwrap()
}

#[test]
fn test_source_check() {
    for file in read_dir(BASE_DIR.as_path()).unwrap() {
        let entry = file.unwrap().path();
        run_check(&entry);
    }
}

fn run_check(file: &Path) {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("check")
        .arg(file)
        .output();
    let output = output.unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(output.stdout.len(), 0);
}
