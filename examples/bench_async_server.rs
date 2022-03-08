use structopt::StructOpt;
use tracing_subscriber::util::SubscriberInitExt;
use ws_tool::{
    codec::{default_handshake_handler, AsyncWsBytesCodec},
    frame::OpCode,
    ServerBuilder,
};

/// websocket client connect to binance futures websocket
#[derive(StructOpt)]
struct Args {
    /// server host
    #[structopt(long, default_value = "127.0.0.1")]
    host: String,
    /// server port
    #[structopt(short, long, default_value = "9000")]
    port: u16,

    /// level
    #[structopt(short, long, default_value = "info")]
    level: tracing::Level,
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let args = Args::from_args();
    tracing_subscriber::fmt::fmt()
        .with_max_level(args.level)
        .finish()
        .try_init()
        .expect("failed to init log");
    tracing::info!("binding on {}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", args.host, args.port))
        .await
        .unwrap();
    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        stream.set_nodelay(true).unwrap();
        tokio::spawn(async move {
            tracing::info!("got connect from {:?}", addr);
            let mut server = ServerBuilder::async_accept(
                stream,
                default_handshake_handler,
                AsyncWsBytesCodec::factory,
            )
            .await
            .unwrap();

            loop {
                if let Ok(msg) = server.receive().await {
                    if msg.code == OpCode::Close {
                        break;
                    }
                    server.send(&msg.data[..]).await.unwrap();
                } else {
                    break;
                }
            }
            tracing::info!("one conn down");
        });
    }
}