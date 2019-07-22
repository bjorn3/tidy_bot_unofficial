use std::fs;
use std::io;
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

    clone_repo(repo, commit).unwrap();

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

macro_rules! cmd {
    (@($wd:expr) $cmd:ident $($args:expr),*) => {{
        let mut cmd = Command::new(stringify!($cmd));
        cmd.current_dir($wd);
        $(
            cmd.arg($args);
        )*
        cmd.status()
    }};
    ($cmd:ident $($args:expr),*) => {
        cmd!(@(".") $cmd $($args),*);
    };
}

pub fn clone_repo(repo: &str, commit: &str) -> io::Result<()> {
    if fs::read_dir("rust").is_err() {
        println!("===> Cloning https://github.com/rust-lang/rust.git");
        cmd!(git "clone", "https://github.com/rust-lang/rust.git")?;
    }

    let repo_id = {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        repo.hash(&mut hasher);
        hasher.finish().to_string()
    };

    println!("===> Cloning {}", repo);

    let known_remotes = String::from_utf8(
        Command::new("git")
            .current_dir("rust")
            .arg("remote")
            .stdout(Stdio::piped())
            .output()?
            .stdout,
    )
    .unwrap();

    if known_remotes
        .split('\n')
        .find(|repo| repo == &repo_id)
        .is_none()
    {
        cmd!(@("rust") git "remote", "add", &repo_id, repo)?;
    }
    cmd!(@("rust") git "fetch", repo_id)?;
    cmd!(@("rust") git "checkout", commit)?;

    println!("===> Checked out {}", commit);

    Ok(())
}
