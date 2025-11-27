use std::process::Command;

/// Get the path to the idgen binary
fn idgen_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove 'deps' directory
    path.push("idgen");

    // On Windows, add .exe extension
    #[cfg(target_os = "windows")]
    path.set_extension("exe");

    path
}

// ============================================
// Success Exit Code (0) Tests
// ============================================

#[test]
fn test_exit_code_success_default() {
    let output = Command::new(idgen_bin())
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_success_uuid_v4() {
    let output = Command::new(idgen_bin())
        .args(["-t", "uuid4"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_success_nanoid() {
    let output = Command::new(idgen_bin())
        .args(["-t", "nanoid"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_success_ulid() {
    let output = Command::new(idgen_bin())
        .args(["-t", "ulid"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_success_objectid() {
    let output = Command::new(idgen_bin())
        .args(["-t", "objectid"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_success_cuid1() {
    let output = Command::new(idgen_bin())
        .args(["-t", "cuid1"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_success_cuid2() {
    let output = Command::new(idgen_bin())
        .args(["-t", "cuid2"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_success_help() {
    let output = Command::new(idgen_bin())
        .args(["--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_success_version() {
    let output = Command::new(idgen_bin())
        .args(["--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_success_json() {
    let output = Command::new(idgen_bin())
        .args(["--json"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_success_inspect_valid_uuid() {
    let output = Command::new(idgen_bin())
        .args(["inspect", "550e8400-e29b-44d4-a716-446655440000"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_success_uuid_v3_with_params() {
    let output = Command::new(idgen_bin())
        .args(["-t", "uuid3", "--namespace", "DNS", "--name", "test"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_success_uuid_v5_with_params() {
    let output = Command::new(idgen_bin())
        .args(["-t", "uuid5", "--namespace", "URL", "--name", "test"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

// ============================================
// Error Exit Code (1) Tests - Runtime Errors
// ============================================

#[test]
fn test_exit_code_error_inspect_invalid_id() {
    let output = Command::new(idgen_bin())
        .args(["inspect", "not-a-valid-id"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
}

// ============================================
// Usage Error Exit Code (2) Tests - Invalid Arguments
// ============================================

#[test]
fn test_exit_code_usage_error_uuid_v3_missing_namespace() {
    let output = Command::new(idgen_bin())
        .args(["-t", "uuid3", "--name", "test"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.to_lowercase().contains("namespace"));
}

#[test]
fn test_exit_code_usage_error_uuid_v3_missing_name() {
    let output = Command::new(idgen_bin())
        .args(["-t", "uuid3", "--namespace", "DNS"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.to_lowercase().contains("name"));
}

#[test]
fn test_exit_code_usage_error_uuid_v3_missing_both() {
    let output = Command::new(idgen_bin())
        .args(["-t", "uuid3"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_exit_code_usage_error_uuid_v5_missing_namespace() {
    let output = Command::new(idgen_bin())
        .args(["-t", "uuid5", "--name", "test"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_exit_code_usage_error_uuid_v5_missing_name() {
    let output = Command::new(idgen_bin())
        .args(["-t", "uuid5", "--namespace", "DNS"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_exit_code_usage_error_invalid_namespace_format() {
    let output = Command::new(idgen_bin())
        .args(["-t", "uuid3", "--namespace", "x", "--name", "test"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_exit_code_usage_error_count_zero() {
    let output = Command::new(idgen_bin())
        .args(["-c", "0"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Count must be at least 1"));
}

// ============================================
// Multiple IDs - Success
// ============================================

#[test]
fn test_exit_code_success_multiple_ids() {
    let output = Command::new(idgen_bin())
        .args(["-c", "5"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have 5 lines (5 UUIDs)
    assert_eq!(stdout.lines().count(), 5);
}

// ============================================
// Prefix/Suffix - Success
// ============================================

#[test]
fn test_exit_code_success_with_prefix() {
    let output = Command::new(idgen_bin())
        .args(["-p", "test-"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("test-"));
}

#[test]
fn test_exit_code_success_with_suffix() {
    let output = Command::new(idgen_bin())
        .args(["-s", ".log"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().ends_with(".log"));
}

// ============================================
// Shell Completions - Success
// ============================================

#[test]
fn test_exit_code_success_completions_bash() {
    let output = Command::new(idgen_bin())
        .args(["completions", "bash"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("_idgen"));
}

#[test]
fn test_exit_code_success_completions_zsh() {
    let output = Command::new(idgen_bin())
        .args(["completions", "zsh"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_success_completions_fish() {
    let output = Command::new(idgen_bin())
        .args(["completions", "fish"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_success_completions_powershell() {
    let output = Command::new(idgen_bin())
        .args(["completions", "powershell"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}
