
use std::process::Command;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use lazy_static;
use tempfile::TempDir;


lazy_static::lazy_static! {
    static ref BASE_DIR: PathBuf = Path::new("tests").join("simpla_programs");
}


#[test]
fn test_source_c_translation() {
    let target_dir = TempDir::new().unwrap();

    for entry in read_dir(BASE_DIR.as_path()).unwrap() {
        let file = entry.unwrap().path();
        let (c_source, target) = get_c_file_name(&file, &target_dir.path());
        let output = Command::new("cargo").arg("run").arg("--").arg("compile").arg(file).arg(&c_source).output();
        assert!(output.unwrap().status.success());
        let output = Command::new("cc").arg(&c_source).arg("-o").arg(target).output();
        assert!(output.unwrap().status.success());
    }



}

fn get_c_file_name(file: &Path, root: &Path) -> (PathBuf, PathBuf) {
    let file_name = file.file_name().unwrap().to_str().unwrap();
    let index = file_name.find('.').unwrap();
    let base_name = &file_name[..index];
    let name = format!("{}.c", base_name);
    (root.join(name), root.join(base_name))
}



#[test]
fn test_source_check() {
    for file in read_dir(BASE_DIR.as_path()).unwrap() {
        let entry = file.unwrap().path();
        run_check(&entry);
    }
}


fn run_check(file: &Path) {
    let output = Command::new("cargo").arg("run").arg("--").arg("check").arg(file).output();
    let output =output.unwrap();    
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(output.stdout.len(), 0);
}







