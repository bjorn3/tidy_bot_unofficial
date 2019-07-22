use futures::Future;

#[derive(serde::Serialize)]
pub struct CheckRun {
    pub name: &'static str,
    pub head_sha: String,
    pub status: &'static str,
    pub conclusion: &'static str,
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
    pub annotation_level: &'static str,
    pub message: String,
}

impl CheckRun {
    pub fn submit(
        self,
        client: &reqwest::r#async::Client,
        installation: &crate::gh::installation::Installation,
        repo_full_name: String,
    ) -> impl Future<Item = (), Error = reqwest::Error> {
        let client = client.clone();
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
                            println!("check run submit res: status {:?}", status);
                            println!("{}", body);
                        })
                    })
            })
    }
}
