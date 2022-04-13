// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::path::Path;

use itertools::Itertools;

use crate::common::TestEnvironment;

pub mod common;

fn create_commit(test_env: &TestEnvironment, repo_path: &Path, name: &str, parents: &[&str]) {
    if parents.is_empty() {
        test_env.jj_cmd_success(repo_path, &["co", "root"]);
    } else if parents.len() == 1 {
        test_env.jj_cmd_success(repo_path, &["co", parents[0]]);
    } else {
        let mut args = vec!["merge", "-m", name];
        args.extend(parents);
        test_env.jj_cmd_success(repo_path, &args);
        test_env.jj_cmd_success(repo_path, &["co", &format!(r#"description("{name}")"#)]);
        test_env.jj_cmd_success(repo_path, &["open", "@-"]);
        test_env.jj_cmd_success(repo_path, &["co", "@-"]);
    }
    std::fs::write(repo_path.join(name), &format!("{name}\n")).unwrap();
    test_env.jj_cmd_success(repo_path, &["branch", name]);
    test_env.jj_cmd_success(repo_path, &["close", "-m", name]);
}

#[test]
fn test_rebase_invalid() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_success(test_env.env_root(), &["init", "repo", "--git"]);
    let repo_path = test_env.env_root().join("repo");

    create_commit(&test_env, &repo_path, "a", &[]);
    create_commit(&test_env, &repo_path, "b", &["a"]);

    // Missing destination
    let stderr = test_env.jj_cmd_failure(&repo_path, &["rebase"]);
    insta::assert_snapshot!(stderr.lines().take(3).join("\n"), @r###"
    error: The following required arguments were not provided:
        --destination <DESTINATION>
    "###);

    // Both -r and -s
    let stderr = test_env.jj_cmd_failure(&repo_path, &["rebase", "-r", "a", "-s", "a", "-d", "b"]);
    insta::assert_snapshot!(stderr.lines().next().unwrap(), @"error: The argument '--revision <REVISION>' cannot be used with '--source <SOURCE>'");

    // Rebase onto descendant with -r
    let stderr = test_env.jj_cmd_failure(&repo_path, &["rebase", "-r", "a", "-d", "b"]);
    insta::assert_snapshot!(stderr, @"Error: Cannot rebase 247da0ddee3d onto descendant 18db23c14b3c
");

    // Rebase onto descendant with -s
    let stderr = test_env.jj_cmd_failure(&repo_path, &["rebase", "-s", "a", "-d", "b"]);
    insta::assert_snapshot!(stderr, @"Error: Cannot rebase 247da0ddee3d onto descendant 18db23c14b3c
");
}

#[test]
fn test_rebase_single_revision() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_success(test_env.env_root(), &["init", "repo", "--git"]);
    let repo_path = test_env.env_root().join("repo");

    create_commit(&test_env, &repo_path, "a", &[]);
    create_commit(&test_env, &repo_path, "b", &[]);
    create_commit(&test_env, &repo_path, "c", &["a", "b"]);
    create_commit(&test_env, &repo_path, "d", &["c"]);
    // Test the setup
    let template = r#"branches"#;
    let stdout = test_env.jj_cmd_success(&repo_path, &["log", "-T", template]);
    insta::assert_snapshot!(stdout, @r###"
    @ 
    o d
    o   c
    |\  
    o | b
    | o a
    |/  
    o 
    "###);

    // Descendants of rebased commit should be rebased onto parents. First we test
    // with a non-merge commit, so the descendants should be rebased onto the
    // single parent (commit "a"). Then we test with a merge commit, so the
    // descendants should be rebased onto the two parents.
    test_env.jj_cmd_success(&repo_path, &["rebase", "-r", "b", "-d", "a"]);
    let stdout = test_env.jj_cmd_success(&repo_path, &["log", "-T", template]);
    insta::assert_snapshot!(stdout, @r###"
    @ 
    o d
    o c
    | o b
    | o a
    |/  
    o 
    "###);
    test_env.jj_cmd_success(&repo_path, &["undo"]);
    test_env.jj_cmd_success(&repo_path, &["rebase", "-r", "c", "-d", "root"]);
    let stdout = test_env.jj_cmd_success(&repo_path, &["log", "-T", template]);
    insta::assert_snapshot!(stdout, @r###"
    @ 
    o   d
    |\  
    | | o c
    o | | b
    | |/  
    |/|   
    | o a
    |/  
    o 
    "###);
}

#[test]
fn test_rebase_multiple_destinations() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_success(test_env.env_root(), &["init", "repo", "--git"]);
    let repo_path = test_env.env_root().join("repo");

    create_commit(&test_env, &repo_path, "a", &[]);
    create_commit(&test_env, &repo_path, "b", &[]);
    create_commit(&test_env, &repo_path, "c", &[]);
    // Test the setup
    let template = r#"branches"#;
    let stdout = test_env.jj_cmd_success(&repo_path, &["log", "-T", template]);
    insta::assert_snapshot!(stdout, @r###"
    @ 
    o c
    | o b
    |/  
    | o a
    |/  
    o 
    "###);

    test_env.jj_cmd_success(&repo_path, &["rebase", "-r", "a", "-d", "b", "-d", "c"]);
    let stdout = test_env.jj_cmd_success(&repo_path, &["log", "-T", template]);
    insta::assert_snapshot!(stdout, @r###"
    o   a
    |\  
    | | @ 
    | |/  
    |/|   
    o | c
    | o b
    |/  
    o 
    "###);
}

#[test]
fn test_rebase_with_descendants() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_success(test_env.env_root(), &["init", "repo", "--git"]);
    let repo_path = test_env.env_root().join("repo");

    create_commit(&test_env, &repo_path, "a", &[]);
    create_commit(&test_env, &repo_path, "b", &[]);
    create_commit(&test_env, &repo_path, "c", &["a", "b"]);
    create_commit(&test_env, &repo_path, "d", &["c"]);
    // Test the setup
    let template = r#"branches"#;
    let stdout = test_env.jj_cmd_success(&repo_path, &["log", "-T", template]);
    insta::assert_snapshot!(stdout, @r###"
    @ 
    o d
    o   c
    |\  
    o | b
    | o a
    |/  
    o 
    "###);

    test_env.jj_cmd_success(&repo_path, &["rebase", "-s", "b", "-d", "a"]);
    let stdout = test_env.jj_cmd_success(&repo_path, &["log", "-T", template]);
    insta::assert_snapshot!(stdout, @r###"
    @ 
    o d
    o c
    o b
    o a
    o 
    "###);
}