use std::process::{Command, Stdio};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref GIT_LOCK: Mutex<()> = Mutex::new(());
}

pub struct Error {
    pub message: String,
    pub file_and_line: Option<(String, u64)>,
}

pub fn run_tidy(repo: &str, commit: &str) -> (String, Vec<Error>) {
    let _git_lock = GIT_LOCK.lock().unwrap_or_else(|err| err.into_inner());

    let clone_status = Command::new("bash")
        .arg("./clone_repo.sh")
        .arg(repo)
        .arg(commit)
        .status()
        .expect("Couldn't run clone_repo.sh");
    if !clone_status.success() {
        panic!("Cloning of {} (commit {}) failed", repo, commit);
    }

    let output = Command::new("../tidy")
        .current_dir("rust")
        .arg("src")
        .arg("cargo")
        .arg("--no-vendor")
        .stdout(Stdio::piped())
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    let result = stdout + &stderr;

    let success_regex = regex::Regex::new(
        r#"\* \d+ error codes
\* highest error code: E\d+
\* \d+ features"#,
    )
    .unwrap();

    if success_regex.is_match(result.trim()) {
        return (result, Vec::new());
    }

    let line_too_long_regex =
        regex::Regex::new(r#"tidy error: ([\w\d/\.-_]+):(\d+): line longer than 100 chars"#)
            .unwrap();

    let mut errors = Vec::new();

    for line in result.lines() {
        if line == "some tidy checks failed" {
        } else if let Some(captures) = line_too_long_regex.captures(line) {
            errors.push(Error {
                message: "line longer than 100 chars".to_string(),
                file_and_line: Some((captures[1].to_string(), captures[2].parse().unwrap())),
            })
        } else {
            errors.push(Error {
                message: line.to_string(),
                file_and_line: None,
            })
        }
    }

    (result, errors)
}
