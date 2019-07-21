use hyper::rt::{Future, Stream};
use hyper::{Body, Request, Response};

const GITHUB_APP_IDENTIFIER: &str = "<...>";

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

            let installation = data["installation"].as_object().unwrap();
            let installation_id = installation["id"].as_u64().unwrap().to_string();

            check_handler(clone_url, head_sha, url, check_runs_url, repo_full_name, installation_id)
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
    installation_id: String,
) -> impl Future<Item = (), Error = hyper::Error> {
    crate::check::clone_repo(clone_url, head_sha).unwrap();

    println!("{} {}", url, check_runs_url);

    for error in crate::check::run_tidy() {
        if let Some((file, line)) = error.file_and_line {
            print!("{:<32} {:<4}: ", file, line);
        } else {
            print!("{:<37}: ", "<unknown>");
        }
        println!("{}", error.message);
    }

    #[derive(serde::Serialize)]
    struct CheckRun {
        name: &'static str,
        head_sha: String,
        status: &'static str,
        conclusion: &'static str,
    }

    let check_run_data = CheckRun {
        name: "tidy",
        head_sha: head_sha.to_string(),
        status: "completed",
        conclusion: "failure", // "success"
                               //output,
                               //actions: Vec::new(),
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let header = serde_json::json!({});
    let payload = serde_json::json!({
        "iat": now,
        "exp": now + 10 * 60,
        "iss": GITHUB_APP_IDENTIFIER.to_string(),
    });

    let token = frank_jwt::encode(
        header,
        &std::path::PathBuf::from("./<...>.pem"),
        &payload,
        frank_jwt::Algorithm::RS256,
    )
    .unwrap();

    println!("token {}", token);

    let client = reqwest::r#async::Client::new();

    let res = client.post(&format!("https://api.github.com/app/installations/{}/access_tokens", installation_id))
        .header("Accept", "application/vnd.github.machine-man-preview+json")
        .header("Authorization", format!("Bearer {}", token))
        .send();

    let install_token = res.and_then(|mut res| res.text()).map(|text| {
        println!("access tokens: {}", text);
        let data = serde_json::from_str::<serde_json::Value>(&text).unwrap();
        let data = data.as_object().unwrap();
        data["token"].as_str().unwrap().to_string()
    });

    install_token.and_then(move |install_token| {
        client
            .post(&format!(
                "https://api.github.com/repos/{repo_full_name}/check-runs",
                repo_full_name = repo_full_name
            ))
            .header("Accept", "application/vnd.github.antiope-preview+json")
            .header("Authorization", format!("Bearer {}", install_token))
            .json(&check_run_data)
            .send()
            .and_then(|mut res| {
                let status = res.status();
                res.text().map(move |body| {
                    println!("check run submit res: status {:?}", status);
                    println!("{}", body);
                })
            })

    }).map_err(|err| {
                panic!("err: {:?}", err);
            })
}
