use fsdr_blocks::stream::MultistreamSink;
use futuresdr::anyhow::Result;
use futuresdr::blocks::SignalSourceBuilder;
use futuresdr::macros::connect;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

use axum::body::{Bytes, StreamBody};
use axum::response::Html;
use axum::routing::get;
use axum::Extension;
use axum::Router;
use futuresdr::futures::stream::Stream;
use std::io;
// use futures::channel::mpsc;
// use futures::channel::oneshot;
use std::sync::{Arc, Mutex};
use tower_http::add_extension::AddExtensionLayer;
use tower_http::cors::CorsLayer;

fn main() -> Result<()> {
    const RATE: f32 = 48_000.0;
    let mut fg = Flowgraph::new();

    // A simple tone generator
    let src = SignalSourceBuilder::<f32>::sin(440.0, RATE)
        .amplitude(0.3)
        .build();

    // A simple sink that send bytes into audio stream
    // to be played inside your browser
    let streams = Vec::<futures::channel::mpsc::Sender<f32>>::new();
    let streams = Arc::new(Mutex::new(streams));
    let streaming_sink = MultistreamSink::<f32>::new(Arc::clone(&streams));

    connect!(fg,
        src > streaming_sink;
    );

    let router = Router::new()
        .route("/my_route/", get(my_route))
        .route("/stream.txt", get(handler_my_sound))
        .route("/stream.wav", get(handler_my_sound))
        .layer(AddExtensionLayer::new(Arc::clone(&streams)))
        .layer(CorsLayer::permissive());

    println!("Visit http://localhost:1337/my_route/");

    Runtime::with_custom_routes(router).run(fg)?;

    Ok(())
}

async fn my_route() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
    <html>
        <head>
            <meta charset='utf-8' />
            <title>FutureSDR</title>
        </head>
        <body>
            <h1>Audio streaming example</h1>
            Visit <a href="/stream.txt">stream</a>
            <audio controls>
                <source src="/stream.wav" type="audio/wav" preload="none">
                Your browser does not support the audio element.
            </audio>
        </body>
    </html>
    "#,
    )
}

struct DropReceiver {
    // receiver: Arc<Mutex<Receiver<f32>>>,
}

impl Drop for DropReceiver {
    /// Clean closure of channel
    /// https://docs.rs/tokio/0.1.22/tokio/sync/mpsc/index.html#clean-shutdown
    fn drop(&mut self) {
        println!("1 connexion closed!");
        // let mut receiver = self.receiver.lock().unwrap();
        // receiver.close();
    }
}

async fn handler_my_sound(
    Extension(streams): Extension<Arc<Mutex<Vec<futures::channel::mpsc::Sender<f32>>>>>,
) -> StreamBody<impl Stream<Item = io::Result<Bytes>>> {
    let _dropper = DropReceiver { /*receiver: Arc::clone(&rec) */};
    let stream = MultistreamSink::<f32>::as_riff_wav_stream(streams, 1_000);
    StreamBody::new(stream)
}
