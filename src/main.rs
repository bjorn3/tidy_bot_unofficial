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
            let res_body_fut = body
                .collect()
                .and_then(move |chunks| {
                    let body = chunks
                        .into_iter()
                        .map(|chunk| chunk.into_bytes().into_iter())
                        .flatten()
                        .collect::<Vec<_>>();

                    handlers::handle(parts, body)
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
