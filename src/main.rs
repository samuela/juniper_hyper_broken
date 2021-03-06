// This is a minimally edited copy of https://github.com/graphql-rust/juniper/blob/master/juniper_hyper/examples/hyper_server.rs.

use std::sync::Arc;

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Response, Server, StatusCode,
};
use juniper::{
    tests::fixtures::starwars::{model::Database, schema::Query},
    EmptyMutation, EmptySubscription, RootNode,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let addr = ([127, 0, 0, 1], 3000).into();

    let db = Arc::new(Database::new());
    let root_node = Arc::new(RootNode::new(
        Query,
        EmptyMutation::<Database>::new(),
        // Subscriptions haven't landed in the latest release AFAICT, so we drop it for now. That's ok.
        // EmptySubscription::<Database>::new(),
    ));

    let new_service = make_service_fn(move |_| {
        let root_node = root_node.clone();
        let ctx = db.clone();

        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let root_node = root_node.clone();
                let ctx = ctx.clone();
                async move {
                    match (req.method(), req.uri().path()) {
                        (&Method::GET, "/") => juniper_hyper::graphiql("/graphql", None).await,
                        (&Method::GET, "/graphql") | (&Method::POST, "/graphql") => {
                            juniper_hyper::graphql(root_node, ctx, req).await
                        }
                        _ => {
                            let mut response = Response::new(Body::empty());
                            *response.status_mut() = StatusCode::NOT_FOUND;
                            Ok(response)
                        }
                    }
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(new_service);
    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e)
    }
}
