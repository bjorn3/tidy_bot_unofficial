use futures::Future;

#[derive(serde::Serialize)]
#[serde(transparent)]
pub struct CheckRunId(String);

#[derive(serde::Serialize)]
pub struct CheckRun {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<CheckRunId>,

    pub name: &'static str,
    pub head_sha: String,

    /// # Allowed values
    ///
    /// * "queued"
    /// * "in_progress"
    /// * "completed", requires "conclusion" field
    pub status: &'static str,

    /// Must be some when status is "completed"
    ///
    /// # Allowed values
    ///
    /// * "success"
    /// * "failure"
    /// * "neutral"
    /// * "timed_out"
    /// * "action_required", requires "details_url" field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conclusion: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Output>,
}

#[derive(serde::Serialize)]
pub struct Output {
    pub title: &'static str,
    pub summary: String,
    pub text: String,
    pub annotations: Vec<Annotation>,
}

#[derive(serde::Serialize)]
pub struct Annotation {
    pub path: String,
    pub start_line: u64,
    pub end_line: u64,

    /// # Allowed values
    ///
    /// * "notice"
    /// * "warning"
    /// * "failure"
    pub annotation_level: &'static str,

    pub message: String,
}

impl CheckRun {
    pub fn submit(
        self,
        client: &reqwest::r#async::Client,
        installation: &crate::gh::installation::Installation,
        repo_full_name: &str,
    ) -> impl Future<Item = CheckRunId, Error = reqwest::Error> {
        let client = client.clone();
        let repo_full_name = repo_full_name.to_string();
        installation
            .get_installation_access_token(&client)
            .and_then(move |install_access_token| {
                client
                    .post(&format!(
                        "https://api.github.com/repos/{repo_full_name}/check-runs",
                        repo_full_name = repo_full_name
                    ))
                    .header("Accept", "application/vnd.github.antiope-preview+json")
                    .header(
                        "Authorization",
                        format!("Bearer {}", install_access_token.token),
                    )
                    .json(&self)
                    .send()
                    .and_then(|mut res| {
                        let status = res.status();
                        res.text().map(move |body| {
                            //println!("check run submit res: status {:?}", status);
                            //println!("{}", body);
                            assert!(status.is_success(), "{:?}\n{}", status, body);

                            let data = serde_json::from_str::<serde_json::Value>(&body).unwrap();
                            let data = data.as_object().unwrap();
                            CheckRunId(data["id"].as_u64().unwrap().to_string())
                        })
                    })
            })
    }
}
