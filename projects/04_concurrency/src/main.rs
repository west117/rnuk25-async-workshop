use axum::routing::get;
use errors::AppError;
use tokio::net::TcpListener;

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

    let mut output = String::new();

    let pulls = slow_api::list_pulls().await?;
    for pull in pulls {
        use std::fmt::Write;

        let title = slow_api::get_title(pull).await?;
        writeln!(&mut output, "{pull}: {title}")?;
    }

    Ok(output)
}
