use std::{net::SocketAddr, convert::Infallible};

use hyper::{Request, Response, Body, Server, service::{service_fn, make_service_fn}};
use log::info;

pub async fn serve(pack: Vec<u8>) {
    let addr = SocketAddr::from(([0, 0, 0, 0], 25566));

    let server = Server::bind(&addr)    
        .serve(make_service_fn(move |_conn| {
            let pack = pack.to_vec();
            async move { Ok::<_, Infallible>(service_fn(move |req| respond(req, pack.to_vec()))) }
        }));
    
    info!("Starting http server");
    server.await.unwrap();
}

async fn respond(_req: Request<Body>, pack: Vec<u8>) -> Result<Response<Body>, Infallible> {
    info!("Delivered resource pack");
    Ok(Response::new(pack.to_vec().into()))
}