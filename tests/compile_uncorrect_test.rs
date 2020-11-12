use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::process::Command;

use lazy_static;

lazy_static::lazy_static! {
    static ref BASE_DIR: PathBuf = Path::new("tests").join("simpla-uncorrect_programs");
}


#[test]
fn test_run_check_on_uncorrect_program() {
    for entry in read_dir(BASE_DIR.as_path()).unwrap() {
        let file = entry.unwrap().path();
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("check")
            .arg(file)
            .output();
        let output = output.unwrap();
        assert!(!output.status.success());
    }
}






