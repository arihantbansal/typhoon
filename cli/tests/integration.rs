//! Integration tests for the Typhoon CLI.

use {assert_cmd::Command, assert_fs::prelude::*, predicates::prelude::*};

/// Helper to get the typhoon binary command
fn typhoon_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_typhoon"))
}

/// Tests the `typhoon init` command with the default (counter) template
#[test]
fn test_init_counter_template() {
    let temp = assert_fs::TempDir::new().unwrap();
    let project_name = "test-counter";

    // Run typhoon init
    typhoon_cmd()
        .arg("init")
        .arg(project_name)
        .current_dir(&temp)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Successfully created Typhoon program '{project_name}'"
        )));

    // Verify project structure
    let project_dir = temp.child(project_name);
    project_dir
        .child("Cargo.toml")
        .assert(predicate::path::exists());
    project_dir
        .child("src/lib.rs")
        .assert(predicate::path::exists());
    project_dir
        .child("build.rs")
        .assert(predicate::path::exists());
    project_dir
        .child(".gitignore")
        .assert(predicate::path::exists());

    // Verify Cargo.toml contains package name
    let cargo_toml = std::fs::read_to_string(project_dir.child("Cargo.toml").path()).unwrap();
    assert!(cargo_toml.contains(&format!("name = \"{project_name}\"")));
    assert!(cargo_toml.contains("typhoon"));
}

/// Tests the `typhoon init` command with the hello-world template
#[test]
fn test_init_hello_world_template() {
    let temp = assert_fs::TempDir::new().unwrap();
    let project_name = "test-hello";

    // Run typhoon init with hello-world template
    typhoon_cmd()
        .arg("init")
        .arg(project_name)
        .arg("--template")
        .arg("hello-world")
        .current_dir(&temp)
        .assert()
        .success();

    // Verify project structure
    let project_dir = temp.child(project_name);
    project_dir
        .child("Cargo.toml")
        .assert(predicate::path::exists());
    project_dir
        .child("src/lib.rs")
        .assert(predicate::path::exists());

    // hello-world template should NOT have build.rs
    project_dir
        .child("build.rs")
        .assert(predicate::path::missing());
}

/// Tests the `typhoon init --workspace` command
#[test]
fn test_init_workspace() {
    let temp = assert_fs::TempDir::new().unwrap();
    let workspace_name = "test-workspace";

    // Run typhoon init --workspace
    typhoon_cmd()
        .arg("init")
        .arg(workspace_name)
        .arg("--workspace")
        .current_dir(&temp)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Successfully created Typhoon workspace '{workspace_name}'"
        )));

    // Verify workspace structure
    let workspace_dir = temp.child(workspace_name);
    workspace_dir
        .child("Cargo.toml")
        .assert(predicate::path::exists());
    workspace_dir
        .child("programs")
        .assert(predicate::path::is_dir());
    workspace_dir
        .child(".gitignore")
        .assert(predicate::path::exists());

    // Verify Cargo.toml contains workspace section
    let cargo_toml = std::fs::read_to_string(workspace_dir.child("Cargo.toml").path()).unwrap();
    assert!(cargo_toml.contains("[workspace]"));
    assert!(cargo_toml.contains("members"));
}

/// Tests the `typhoon add program` command
#[test]
fn test_add_program_to_workspace() {
    let temp = assert_fs::TempDir::new().unwrap();
    let workspace_name = "test-workspace";
    let program_name = "my-program";

    // First, create a workspace
    typhoon_cmd()
        .arg("init")
        .arg(workspace_name)
        .arg("--workspace")
        .current_dir(&temp)
        .assert()
        .success();

    let workspace_dir = temp.child(workspace_name);

    // Add a program to the workspace
    typhoon_cmd()
        .arg("add")
        .arg("program")
        .arg(program_name)
        .current_dir(&workspace_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Adding program '{program_name}' to workspace"
        )));

    // Verify program was created
    workspace_dir
        .child(format!("programs/{program_name}/Cargo.toml"))
        .assert(predicate::path::exists());
    workspace_dir
        .child(format!("programs/{program_name}/src/lib.rs"))
        .assert(predicate::path::exists());
}

/// Tests the `typhoon add program` command with hello-world template
#[test]
fn test_add_program_hello_world() {
    let temp = assert_fs::TempDir::new().unwrap();
    let workspace_name = "test-workspace";
    let program_name = "hello-program";

    // Create workspace
    typhoon_cmd()
        .arg("init")
        .arg(workspace_name)
        .arg("--workspace")
        .current_dir(&temp)
        .assert()
        .success();

    let workspace_dir = temp.child(workspace_name);

    // Add hello-world program
    typhoon_cmd()
        .arg("add")
        .arg("program")
        .arg(program_name)
        .arg("--template")
        .arg("hello-world")
        .current_dir(&workspace_dir)
        .assert()
        .success();

    // Verify program structure (no build.rs for hello-world)
    workspace_dir
        .child(format!("programs/{program_name}/build.rs"))
        .assert(predicate::path::missing());
}

/// Tests error handling for invalid project names
#[test]
fn test_invalid_project_name_empty() {
    let temp = assert_fs::TempDir::new().unwrap();

    typhoon_cmd()
        .arg("init")
        .arg("")
        .current_dir(&temp)
        .assert()
        .failure();
}

/// Tests error handling for invalid project names (starts with digit)
#[test]
fn test_invalid_project_name_starts_with_digit() {
    let temp = assert_fs::TempDir::new().unwrap();

    typhoon_cmd()
        .arg("init")
        .arg("123project")
        .current_dir(&temp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot start with a digit"));
}

/// Tests error handling for invalid project names (path traversal)
#[test]
fn test_invalid_project_name_path_traversal() {
    let temp = assert_fs::TempDir::new().unwrap();

    typhoon_cmd()
        .arg("init")
        .arg("../evil")
        .current_dir(&temp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("path separators"));
}

/// Tests error handling for invalid project names (Rust keyword)
#[test]
fn test_invalid_project_name_rust_keyword() {
    let temp = assert_fs::TempDir::new().unwrap();

    typhoon_cmd()
        .arg("init")
        .arg("async")
        .current_dir(&temp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Rust keyword"));
}

/// Tests error when trying to create a project that already exists
#[test]
fn test_init_existing_directory() {
    let temp = assert_fs::TempDir::new().unwrap();
    let project_name = "existing-project";

    // Create the first time
    typhoon_cmd()
        .arg("init")
        .arg(project_name)
        .current_dir(&temp)
        .assert()
        .success();

    // Try to create again - should fail
    typhoon_cmd()
        .arg("init")
        .arg(project_name)
        .current_dir(&temp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

/// Tests error when trying to add program outside of workspace
#[test]
fn test_add_program_not_in_workspace() {
    let temp = assert_fs::TempDir::new().unwrap();
    let project_name = "single-program";

    // Create a single program (not workspace)
    typhoon_cmd()
        .arg("init")
        .arg(project_name)
        .current_dir(&temp)
        .assert()
        .success();

    let project_dir = temp.child(project_name);

    // Try to add a program - should fail
    typhoon_cmd()
        .arg("add")
        .arg("program")
        .arg("another-program")
        .current_dir(&project_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not in a workspace"));
}

/// Tests error when trying to add a program that already exists
#[test]
fn test_add_program_already_exists() {
    let temp = assert_fs::TempDir::new().unwrap();
    let workspace_name = "test-workspace";
    let program_name = "duplicate-program";

    // Create workspace
    typhoon_cmd()
        .arg("init")
        .arg(workspace_name)
        .arg("--workspace")
        .current_dir(&temp)
        .assert()
        .success();

    let workspace_dir = temp.child(workspace_name);

    // Add program first time
    typhoon_cmd()
        .arg("add")
        .arg("program")
        .arg(program_name)
        .current_dir(&workspace_dir)
        .assert()
        .success();

    // Try to add again - should fail
    typhoon_cmd()
        .arg("add")
        .arg("program")
        .arg(program_name)
        .current_dir(&workspace_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

/// Tests that `typhoon build` requires being in a Rust project
#[test]
fn test_build_not_in_project() {
    let temp = assert_fs::TempDir::new().unwrap();

    typhoon_cmd()
        .arg("build")
        .current_dir(&temp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cargo.toml not found"));
}

/// Tests that `typhoon test` requires being in a Rust project
#[test]
fn test_test_not_in_project() {
    let temp = assert_fs::TempDir::new().unwrap();

    typhoon_cmd()
        .arg("test")
        .current_dir(&temp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cargo.toml not found"));
}

/// Tests error handling for invalid template names
#[test]
fn test_init_invalid_template() {
    let temp = assert_fs::TempDir::new().unwrap();

    typhoon_cmd()
        .arg("init")
        .arg("test-project")
        .arg("--template")
        .arg("nonexistent")
        .current_dir(&temp)
        .assert()
        .failure()
        .stderr(predicate::str::contains("template 'nonexistent' not found"));
}

/// Tests that program keypairs are generated correctly
#[test]
fn test_keypair_generation() {
    let temp = assert_fs::TempDir::new().unwrap();
    let project_name = "keypair-test";

    // Create project
    typhoon_cmd()
        .arg("init")
        .arg(project_name)
        .current_dir(&temp)
        .assert()
        .success();

    let project_dir = temp.child(project_name);

    // Verify keypair was generated
    let keypair_name = project_name.replace('-', "_");
    let keypair_child = project_dir.child(format!("target/deploy/{keypair_name}-keypair.json"));
    keypair_child.assert(predicate::path::exists());

    // Verify the keypair file is valid JSON
    let keypair_path = keypair_child.path();
    let keypair_content = std::fs::read_to_string(keypair_path).unwrap();
    let _: Vec<u8> = serde_json::from_str(&keypair_content).expect("invalid keypair JSON");
}

/// Tests workspace with both programs/ and program/ directory naming
#[test]
fn test_add_program_supports_singular_directory() {
    let temp = assert_fs::TempDir::new().unwrap();
    let workspace_name = "test-workspace";

    // Create workspace normally (with programs/)
    typhoon_cmd()
        .arg("init")
        .arg(workspace_name)
        .arg("--workspace")
        .current_dir(&temp)
        .assert()
        .success();

    let workspace_dir = temp.child(workspace_name);

    // Manually rename programs/ to program/ to test singular support
    let programs_child = workspace_dir.child("programs");
    let program_child = workspace_dir.child("program");
    std::fs::rename(programs_child.path(), program_child.path()).unwrap();

    // Now try to add a program - should work with program/ directory
    typhoon_cmd()
        .arg("add")
        .arg("program")
        .arg("test-program")
        .current_dir(&workspace_dir)
        .assert()
        .success();

    // Verify program was created in program/ directory
    workspace_dir
        .child("program/test-program/Cargo.toml")
        .assert(predicate::path::exists());
}

/// Tests that `typhoon test` command properly validates workspace binaries
#[test]
fn test_command_validates_workspace_binaries() {
    let temp = assert_fs::TempDir::new().unwrap();
    let workspace_name = "test-workspace";

    // Create workspace
    typhoon_cmd()
        .arg("init")
        .arg(workspace_name)
        .arg("--workspace")
        .current_dir(&temp)
        .assert()
        .success();

    let workspace_dir = temp.child(workspace_name);

    // Add a program
    typhoon_cmd()
        .arg("add")
        .arg("program")
        .arg("test-program")
        .current_dir(&workspace_dir)
        .assert()
        .success();

    // Test command should fail because no programs are built yet
    typhoon_cmd()
        .arg("test")
        .current_dir(&workspace_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("no program binaries found"));

    // Note: We can't actually build the program in tests without Solana CLI
    // being installed, but we've verified the validation logic works
}
