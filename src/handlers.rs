use hyper::rt::Future;

pub fn webhook_handle(
    _parts: hyper::http::request::Parts,
    body: Vec<u8>,
) -> Box<Future<Item = String, Error = hyper::Error> + Send> {
    let data = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
    let data = data.as_object().unwrap();
    let action = data["action"].as_str().unwrap();
    println!("===> action {}", action);
    match action {
        "requested" => {
            let check_suite = data["check_suite"].as_object().unwrap();
            let head_sha = check_suite["head_sha"].as_str().unwrap().to_string();

            let repository = data["repository"].as_object().unwrap();
            let html_url = repository["html_url"].as_str().unwrap().to_string();
            let repo_full_name = repository["full_name"].as_str().unwrap().to_string();

            let installation = serde_json::from_value(data["installation"].clone()).unwrap();

            check_handler(html_url, head_sha, repo_full_name, installation)
                .map(|()| "".to_string())
                .boxed()
        }
        _ => futures::future::ok("".to_string()).boxed(),
    }
}

fn check_handler(
    html_url: String,
    head_sha: String,
    repo_full_name: String,
    installation: crate::gh::installation::Installation,
) -> impl Future<Item = (), Error = hyper::Error> {
    use crate::gh::check::*;
    let client = reqwest::r#async::Client::new();

    CheckRun {
        name: "tidy",
        head_sha: head_sha.to_string(),
        status: "in_progress",
        conclusion: None,
        output: None,
    }
    .submit(&client, &installation, &repo_full_name, None)
    .and_then(move |check_run_id| {
        let (tidy_result, errors) = crate::check::run_tidy(&html_url, &head_sha);

        for error in &errors {
            if let Some((file, line)) = &error.file_and_line {
                print!("{:<32} {:<4}: ", file, line);
            } else {
                print!("{:<37}: ", "<unknown>");
            }
            println!("{}", error.message);
        }

        let annotations = errors
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
            .collect();

        let check_run = CheckRun {
            name: "tidy",
            head_sha: head_sha.to_string(),
            status: "completed",
            conclusion: Some(if errors.is_empty() {
                "success"
            } else {
                "failure"
            }),
            output: Some(Output {
                title: "tidy errors",
                summary: format!("Tidy noticed {} errors", errors.len()),
                text: format!("```plain\n{}\n```", tidy_result.trim()),
                annotations,
            }),
        };

        check_run
            .submit(&client, &installation, &repo_full_name, Some(&check_run_id))
            .map(|_check_run_id| ())
    })
    .map_err(|err| {
        panic!("err: {:?}", err);
    })
}
