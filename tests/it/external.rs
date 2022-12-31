use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).to_owned()
}

#[test]
fn cargo_fmt_check() {
    let output = Command::new("cargo")
        .args(["fmt", "--", "--check"])
        .current_dir(project_root())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    if !output.status.success() {
        Command::new("cargo")
            .arg("fmt")
            .current_dir(project_root())
            .output()
            .unwrap();
        panic!("code wasn't formatted");
    }
}

#[test]
fn cargo_deny_check() {
    let output = Command::new("cargo")
        .args(["deny", "check"])
        .current_dir(project_root())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    assert!(output.status.success());
}
