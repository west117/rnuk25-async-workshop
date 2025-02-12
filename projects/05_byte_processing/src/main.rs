#![allow(unused)]
use axum::{body::Body, routing::get};
use errors::AppError;

use tokio::net::TcpListener;

use futures::{SinkExt, TryStreamExt};
use std::pin::pin;
use tokio_util::{
    codec::{FramedRead, FramedWrite},
    io::{ReaderStream, StreamReader},
};

mod codec;
mod errors;
mod slow_api;

use codec::JsonLinesCodec;
use slow_api::PrTitle;

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

async fn req_handler() -> Result<Body, AppError> {
    println!("received http request");

    let prs_resp = slow_api::get_prs().await?;

    // TODO:
    // translate the get_prs response (lines of PrTitle in json)
    // into lines of String in json,
    // where each string is `format!("{id}: {title}")`.
    // Ideally, do so in a streaming fasion.

    Ok(Body::empty())
}
