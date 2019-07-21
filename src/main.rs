use hyper::rt::{Future, Stream};
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};

mod check;
mod handlers;

fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();

    let new_svc = || {
        service_fn_ok(|req: Request<Body>| {
            let (parts, body) = req.into_parts();
            let res_body_fut = body
                .collect()
                .and_then(move |chunks| {
                    println!("{:#?}", parts);
                    let body = chunks
                        .into_iter()
                        .map(|chunk| chunk.into_bytes().into_iter())
                        .flatten()
                        .collect::<Vec<_>>();
                    println!("{}", String::from_utf8_lossy(&body).to_owned());

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

    hyper::rt::run(server);
}
