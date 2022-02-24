use structopt::StructOpt;
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;
use ws_tool::{
    codec::{default_handshake_handler, AsyncWsStringCodec},
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
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::DEBUG)
        .finish()
        .try_init()
        .expect("failed to init log");
    let args = Args::from_args();
    tracing::info!("binding on {}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", args.host, args.port))
        .await
        .unwrap();
    for (stream, addr) in listener.accept().await {
        tracing::info!("got connect from {:?}", addr);
        let mut server = ServerBuilder::async_accept(
            stream,
            default_handshake_handler,
            AsyncWsStringCodec::factory,
        )
        .await
        .unwrap();

        loop {
            if let Ok((code, msg)) = server.receive().await {
                if code == OpCode::Close {
                    break;
                }
                server.send((None, msg)).await.unwrap();
            } else {
                break;
            }
        }
    }
    Ok(())
}
