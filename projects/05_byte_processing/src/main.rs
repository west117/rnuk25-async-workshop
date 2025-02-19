#![allow(unused)]
use axum::{body::Body, routing::get};
use errors::AppError;

use tokio::{io::simplex, net::TcpListener};

use futures::{SinkExt, TryStreamExt};
use std::{
    io::{self, Error},
    pin::pin,
};
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

    let stream = prs_resp
        .into_body()
        .into_data_stream()
        .map_err(io::Error::other);

    let reader = StreamReader::new(stream);

    let (rx, tx) = simplex(1024);

    tokio::spawn(async move {
        let mut frameread = pin!(FramedRead::new(reader, JsonLinesCodec::<PrTitle>::new()));
        let mut tx = FramedWrite::new(tx, JsonLinesCodec::<String>::new());

        while let Some(msg) = frameread.as_mut().try_next().await.unwrap() {
            let PrTitle { id, title } = msg;
            let out_msg = format!("{id}: {title}");

            tx.send(out_msg).await.unwrap();
        }
        tx.close().await.unwrap();
    });

    let rx = ReaderStream::new(rx);

    Ok(Body::from_stream(rx))
}
