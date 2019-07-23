#![recursion_limit = "128"]

use hyper::rt::{Future, Stream};
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};

mod check;
mod handlers;

mod gh {
    pub mod check;
    pub mod installation;
}

fn main() {
    let addr = (
        [0, 0, 0, 0],
        std::env::var("PORT")
            .unwrap_or("3000".to_string())
            .parse::<u16>()
            .unwrap(),
    )
        .into();

    let new_svc = || {
        service_fn_ok(|req: Request<Body>| {
            let (parts, body) = req.into_parts();
            if parts.method == hyper::http::Method::GET {
                return Response::new(Body::from(
                    "Rust tidy bot (unofficial).\n\
                     See https://github.com/bjorn3/tidy_bot_unofficial for more information."
                        .to_string(),
                ));
            }
            let res_body_fut = body
                .collect()
                .and_then(move |chunks| {
                    let body = chunks
                        .into_iter()
                        .map(|chunk| chunk.into_bytes().into_iter())
                        .flatten()
                        .collect::<Vec<_>>();

                    if let Ok(()) = verify_webhook_hub_signature(&parts, &body) {
                        handlers::webhook_handle(parts, body)
                    } else {
                        futures::future::ok("webhook hub signature invalid".to_string()).boxed()
                    }
                })
                .map_err(|e| {
                    println!("err while receiving req body: {:?}", e);
                    "err"
                });

            Response::new(Body::wrap_stream(res_body_fut.into_stream()))
        })
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Serving at {:?}", addr);

    hyper::rt::run(server);
}

fn verify_webhook_hub_signature(
    parts: &hyper::http::request::Parts,
    body: &[u8],
) -> Result<(), hmac::crypto_mac::MacError> {
    use hmac::Mac;

    let secret_token = if let Ok(secret_token) = std::env::var("GITHUB_SECRET_TOKEN") {
        secret_token
    } else {
        return Ok(());
    };

    let signature = parts
        .headers
        .get("X-HUB-SIGNATURE")
        .expect("X-HUB-SIGNATURE")
        .as_bytes();

    let mut mac = hmac::Hmac::<sha1::Sha1>::new_varkey(secret_token.as_bytes()).unwrap();
    mac.input(&body);
    mac.verify(
        &(b"sha1="
            .iter()
            .cloned()
            .chain(signature.iter().cloned())
            .collect::<Vec<u8>>()),
    ).map_err(|err| {
        println!("Webhook X-HUB-SIGNATURE {:?} failed to verify", signature);
        err
    })
}
