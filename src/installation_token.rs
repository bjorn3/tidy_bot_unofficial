use futures::Future;

const GITHUB_APP_IDENTIFIER: &str = env!("GITHUB_APP_IDENTIFIER");
const GITHUB_PRIVATE_KEY: &str = env!("GITHUB_PRIVATE_KEY");

pub fn get_installation_token(
    client: &reqwest::r#async::Client,
    installation_id: String,
) -> impl Future<Item = String, Error = reqwest::Error> {
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
        &std::path::PathBuf::from(GITHUB_PRIVATE_KEY),
        &payload,
        frank_jwt::Algorithm::RS256,
    )
    .unwrap();

    println!("token {}", token);

    let client = reqwest::r#async::Client::new();

    let res = client
        .post(&format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            installation_id
        ))
        .header("Accept", "application/vnd.github.machine-man-preview+json")
        .header("Authorization", format!("Bearer {}", token))
        .send();

    res.and_then(|mut res| res.text()).map(|text| {
        println!("access tokens: {}", text);
        let data = serde_json::from_str::<serde_json::Value>(&text).unwrap();
        let data = data.as_object().unwrap();
        data["token"].as_str().unwrap().to_string()
    })
}
