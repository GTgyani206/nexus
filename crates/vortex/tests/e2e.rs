use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn write_program(name: &str, source: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("valid system time")
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!("nexus_{name}_{stamp}.vx"));
    fs::write(&path, source).expect("program file should be writable");
    path
}

fn run_nexus(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_nexus"))
        .args(args)
        .output()
        .expect("nexus command should run");

    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn run_ic_arithmetic() {
    let path = write_program("arithmetic", "2 + 3");
    let file = path.to_string_lossy().to_string();
    let (ok, stdout, stderr) = run_nexus(&["run", &file, "--ic"]);

    let _ = fs::remove_file(path);
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains('5'));
}

#[test]
fn run_ic_subtraction_and_multiplication() {
    let path = write_program("math", "10 - 4\n6 * 7");
    let file = path.to_string_lossy().to_string();
    let (ok, stdout, stderr) = run_nexus(&["run", &file, "--ic"]);

    let _ = fs::remove_file(path);
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("42"));
}

#[test]
fn run_ic_let_and_nested_let() {
    let source = "let x = 8\nlet a = 3\nlet b = 4\na * a + b * b";
    let path = write_program("let", source);
    let file = path.to_string_lossy().to_string();
    let (ok, stdout, stderr) = run_nexus(&["run", &file, "--ic"]);

    let _ = fs::remove_file(path);
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("25"));
}

#[test]
fn run_ic_function_call() {
    let source = "fn square(n):\nreturn n * n\nlet result = square(9)\nresult";
    let path = write_program("function", source);
    let file = path.to_string_lossy().to_string();
    let (ok, stdout, stderr) = run_nexus(&["run", &file, "--ic"]);

    let _ = fs::remove_file(path);
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("81"));
}

#[test]
fn run_legacy_path_still_executes() {
    let path = write_program("legacy", "2 + 3");
    let file = path.to_string_lossy().to_string();
    let (ok, stdout, stderr) = run_nexus(&["run", &file]);

    let _ = fs::remove_file(path);
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains('5'));
}

#[test]
fn net_command_dumps_debug_view() {
    let path = write_program("net", "2 + 3");
    let file = path.to_string_lossy().to_string();
    let (ok, stdout, stderr) = run_nexus(&["net", &file]);

    let _ = fs::remove_file(path);
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("cells"));
}
