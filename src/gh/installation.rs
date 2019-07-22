use futures::Future;

const GITHUB_APP_IDENTIFIER: &str = env!("GITHUB_APP_IDENTIFIER");
const GITHUB_PRIVATE_KEY: &str = env!("GITHUB_PRIVATE_KEY");

#[derive(serde::Deserialize)]
pub struct Installation {
    id: u64,
    node_id: String,
    // no more fields
}

#[derive(serde::Deserialize)]
pub struct InstallationAccessToken {
    pub token: String,
    pub expires_at: String, // "2019-07-22T10:08:26Z"
    pub permissions: std::collections::HashMap<String, String>, // { "checks": "write" }
}

impl Installation {
    pub fn get_installation_access_token(
        &self,
        client: &reqwest::r#async::Client,
    ) -> impl Future<Item = InstallationAccessToken, Error = reqwest::Error> {
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

        //println!("JWT token {}", token);

        let res = client
            .post(&format!(
                "https://api.github.com/app/installations/{}/access_tokens",
                self.id
            ))
            .header("Accept", "application/vnd.github.machine-man-preview+json")
            .header("Authorization", format!("Bearer {}", token))
            .send();

        res.and_then(|mut res| {
            let status = res.status();
            res.text().map(move |text| {
                assert!(status.is_success(), "{:?}\n{}", status, text);
                println!("access tokens: {}", text);
                serde_json::from_str::<InstallationAccessToken>(&text).unwrap()
            })
        })
    }
}
