use axum::routing::get;
use errors::AppError;
use tokio::{net::TcpListener, task::JoinSet};

mod errors;
mod slow_api;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = tokio::spawn(http_server());

    server.await??;

    Ok(())
}

async fn http_server() -> anyhow::Result<()> {
    let router = axum::Router::new().route("/", get(req_handler));
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, router).await?;
    Ok(())
}

async fn req_handler() -> Result<String, AppError> {
    println!("received http request");

    let pulls = slow_api::list_pulls().await?;

    let mut joinset = JoinSet::new();

    for pull in pulls {
        joinset.spawn(async move {
            let title = slow_api::get_title(pull).await.unwrap();
            format!("{pull}: {title}")
        });
    }

    let mut responses = vec![];
    while let Some(Ok(resp)) = joinset.join_next().await {
        responses.push(resp);
    }

    Ok(responses.join("\n"))
}
