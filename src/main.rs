mod options;

use log::info;
use log::warn;
use options::ClientClientMode;
use options::Options;
use std::error::Error;
use std::time::Duration;
use std::unimplemented;
use structopt::StructOpt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

const BUFFER_SIZE: usize = 8192;

async fn copy_data(mut read: OwnedReadHalf, mut write: OwnedWriteHalf) -> Result<(), Box<String>> {
    let mut buffer = [0; BUFFER_SIZE];

    loop {
        match read.read(&mut buffer).await {
            Ok(0) => break,
            Ok(length) => match write.write_all(&buffer[0..length]).await {
                Ok(_) => {}
                Err(error) => {
                    warn!("Failed to write to socket: {}", error);

                    break;
                }
            },
            Err(error) => {
                warn!("Failed to read from socket: {}", error);

                break;
            }
        }
    }

    Ok(())
}

async fn server_server(
    external_address: &str,
    internal_address: &str,
) -> Result<(), Box<dyn Error>> {
    info!("Server to server mode");
    info!("External address {}", external_address);
    info!("Internal address {}", internal_address);

    let external_listener = TcpListener::bind(external_address).await?;
    let internal_listener = TcpListener::bind(internal_address).await?;

    loop {
        info!("Waiting connections...");

        let (external, internal) =
            tokio::join!(external_listener.accept(), internal_listener.accept());
        let (external_stream, external_address) = external?;
        let (internal_stream, internal_address) = internal?;

        external_stream.set_nodelay(true)?;
        internal_stream.set_nodelay(true)?;

        let (external_read, external_write) = external_stream.into_split();
        let (internal_read, internal_write) = internal_stream.into_split();

        info!(
            "Created channel: {} <---> {}",
            external_address, internal_address,
        );

        let left_pipe = tokio::spawn(async move { copy_data(external_read, internal_write).await });
        let right_pipe =
            tokio::spawn(async move { copy_data(internal_read, external_write).await });

        let _ = left_pipe.await?;
        let _ = right_pipe.await?;

        info!("Channel closed");
    }
}

async fn read_and_connect(
    read_stream: &mut TcpStream,
    remote_address: &str,
) -> Result<Option<TcpStream>, Box<dyn Error>> {
    let mut buffer = [0; BUFFER_SIZE];
    let length = read_stream.read(&mut buffer).await?;
    let result = if length == 0 {
        None
    } else {
        let mut redirect_stream = TcpStream::connect(remote_address).await?;

        redirect_stream.write_all(&buffer[0..length]).await?;

        Some(redirect_stream)
    };

    Ok(result)
}

async fn client_client_one(
    connect_address: &str,
    redirect_address: &str,
    timeout: u64,
) -> Result<(), Box<dyn Error>> {
    let duration = Duration::from_secs(timeout);

    info!("Server to server mode");
    info!("Connect address {}", connect_address);
    info!("Redirect address {}", redirect_address);

    loop {
        info!("Connecting to server ({})...", connect_address);

        let mut connect_stream = match TcpStream::connect(connect_address).await {
            Ok(stream) => stream,
            Err(error) => {
                warn!("Failed to connect server: {}.", error);

                tokio::time::sleep(duration).await;

                continue;
            }
        };
        connect_stream.set_nodelay(true)?;

        info!("Waiting for data...");

        let redirect_stream = match read_and_connect(&mut connect_stream, redirect_address).await {
            Ok(Some(stream)) => stream,
            Ok(None) => continue,
            Err(error) => {
                warn!("Failed to redirect data (connect -> redirect): {}.", error);

                continue;
            }
        };
        redirect_stream.set_nodelay(true)?;

        let (connect_read, connect_write) = connect_stream.into_split();
        let (redirect_read, redirect_write) = redirect_stream.into_split();

        info!(
            "Created channel: {} <---> {}",
            connect_address, redirect_address,
        );

        let left_pipe = tokio::spawn(async move { copy_data(connect_read, redirect_write).await });
        let right_pipe = tokio::spawn(async move { copy_data(redirect_read, connect_write).await });

        let _ = left_pipe.await?;
        let _ = right_pipe.await?;

        info!("Channel closed");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let options = Options::from_args();

    match options {
        Options::ServerServer {
            external_address,
            internal_address,
        } => server_server(&external_address, &internal_address).await,
        Options::ClientClient {
            external_address,
            internal_address,
            timeout,
            mode: ClientClientMode::ConnectExternal,
        } => client_client_one(&external_address, &internal_address, timeout).await,
        Options::ClientClient {
            external_address,
            internal_address,
            timeout,
            mode: ClientClientMode::ConnectInternal,
        } => client_client_one(&internal_address, &external_address, timeout).await,
        _ => unimplemented!(),
    }
}
