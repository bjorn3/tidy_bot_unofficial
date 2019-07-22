use hyper::rt::{Future, Stream};
use hyper::{Body, Request, Response};

pub fn handle(
    parts: hyper::http::request::Parts,
    body: Vec<u8>,
) -> impl Future<Item = String, Error = hyper::Error> {
    let data = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let data = data.as_object().unwrap();
    let action = data["action"].as_str().unwrap();
    match action {
        "requested" => {
            let check_suite = data["check_suite"].as_object().unwrap();
            let head_sha = check_suite["head_sha"].as_str().unwrap();
            let url = check_suite["url"].as_str().unwrap();
            let check_runs_url = check_suite["check_runs_url"].as_str().unwrap();

            let repository = data["repository"].as_object().unwrap();
            let clone_url = repository["clone_url"].as_str().unwrap();
            let repo_full_name = repository["full_name"].as_str().unwrap().to_string();

            let installation = serde_json::from_value(data["installation"].clone()).unwrap();

            check_handler(
                clone_url,
                head_sha,
                url,
                check_runs_url,
                repo_full_name,
                installation,
            )
            .map(|()| "success".to_string())
            .boxed()
        }
        _ => {
            println!("action {}", action);
            futures::future::ok("success".to_string()).boxed()
        }
    }
}

fn check_handler(
    clone_url: &str,
    head_sha: &str,
    url: &str,
    check_runs_url: &str,
    repo_full_name: String,
    installation: crate::gh::installation::Installation,
) -> impl Future<Item = (), Error = hyper::Error> {
    crate::check::clone_repo(clone_url, head_sha).unwrap();

    println!("{} {}", url, check_runs_url);

    let errors = crate::check::run_tidy();

    for error in &errors {
        if let Some((file, line)) = &error.file_and_line {
            print!("{:<32} {:<4}: ", file, line);
        } else {
            print!("{:<37}: ", "<unknown>");
        }
        println!("{}", error.message);
    }

    use crate::gh::check::*;

    let check_run_data = CheckRun {
        name: "tidy",
        head_sha: head_sha.to_string(),
        status: "completed",
        conclusion: "failure", // "success"
        output: Output {
            title: "tidy errors",
            summary: format!("Tidy noticed {} errors", errors.len()),
            text: format!(
                "```plain\n{}\n```",
                errors
                    .iter()
                    .filter_map(|error| {
                        if error.file_and_line.is_none() {
                            Some(error.message.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            annotations: errors
                .iter()
                .filter_map(|error| {
                    error
                        .file_and_line
                        .as_ref()
                        .map(|&(ref file, line)| Annotation {
                            path: file.clone(),
                            start_line: line,
                            end_line: line,
                            annotation_level: "failure",
                            message: error.message.clone(),
                        })
                })
                .collect(),
        },
    };

    let client = reqwest::r#async::Client::new();

    check_run_data
        .submit(&client, &installation, repo_full_name)
        .map_err(|err| {
            panic!("err: {:?}", err);
        })
}
