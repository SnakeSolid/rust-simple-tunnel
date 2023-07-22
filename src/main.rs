mod options;

use log::info;
use log::warn;
use options::ClientClientMode;
use options::ClientServerMode;
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

macro_rules! warn_and_break {
    ($($arg:tt)+) => {{
        warn!($($arg)+);

        break;
    }};
}

macro_rules! warn_and_continue {
    ($($arg:tt)+) => {{
        warn!($($arg)+);

        continue;
    }};
}

async fn copy_data(mut read: OwnedReadHalf, mut write: OwnedWriteHalf) -> Result<(), Box<String>> {
    let mut buffer = [0; BUFFER_SIZE];

    loop {
        match read.read(&mut buffer).await {
            Ok(0) => break,
            Ok(length) => match write.write_all(&buffer[0..length]).await {
                Ok(_) => {}
                Err(error) => warn_and_break!("Failed to write to socket: {}", error),
            },
            Err(error) => warn_and_break!("Failed to read from socket: {}", error),
        }
    }

    Ok(())
}

async fn wait_and_connect(
    read_stream: &mut TcpStream,
    remote_address: &str,
) -> Result<Option<TcpStream>, Box<dyn Error>> {
    let mut buffer = [0; BUFFER_SIZE];
    let length = read_stream.peek(&mut buffer).await?;

    match length {
        0 => Ok(None),
        _ => Ok(Some(TcpStream::connect(remote_address).await?)),
    }
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

        println!(
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

        info!("Waiting for data...");

        let redirect_stream = match wait_and_connect(&mut connect_stream, redirect_address).await {
            Ok(Some(stream)) => stream,
            Ok(None) => continue,
            Err(error) => {
                warn_and_continue!("Failed to redirect data (connect -> redirect): {}.", error)
            }
        };

        connect_stream.set_nodelay(true)?;
        redirect_stream.set_nodelay(true)?;

        let (connect_read, connect_write) = connect_stream.into_split();
        let (redirect_read, redirect_write) = redirect_stream.into_split();

        println!(
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

async fn client_client_both(
    external_address: &str,
    internal_address: &str,
    timeout: u64,
) -> Result<(), Box<dyn Error>> {
    let duration = Duration::from_secs(timeout);

    info!("Server to server mode");
    info!("External address {}", external_address);
    info!("Internal address {}", internal_address);

    loop {
        info!("Connecting to {}...", external_address);

        let external_stream = match TcpStream::connect(external_address).await {
            Ok(stream) => stream,
            Err(error) => {
                warn!("Failed to connect (external): {}.", error);

                tokio::time::sleep(duration).await;

                continue;
            }
        };

        info!("Connecting to {}...", external_address);

        let internal_stream = match TcpStream::connect(internal_address).await {
            Ok(stream) => stream,
            Err(error) => {
                warn!("Failed to connect (external): {}.", error);

                tokio::time::sleep(duration).await;

                continue;
            }
        };

        external_stream.set_nodelay(true)?;
        internal_stream.set_nodelay(true)?;

        let (external_read, external_write) = external_stream.into_split();
        let (internal_read, internal_write) = internal_stream.into_split();

        println!(
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

async fn client_server_listen(
    client_address: &str,
    server_address: &str,
    timeout: u64,
) -> Result<(), Box<dyn Error>> {
    let duration = Duration::from_secs(timeout);

    info!("Server server mode");
    info!("Client address {}", client_address);
    info!("Server address {}", server_address);

    let server_listener = TcpListener::bind(server_address).await?;

    loop {
        info!("Waiting connections...");

        let (server_stream, _address) = server_listener.accept().await?;

        info!("Connecting to client {}...", client_address);

        let client_stream = match TcpStream::connect(client_address).await {
            Ok(stream) => stream,
            Err(error) => {
                warn!("Failed to connect: {}.", error);

                tokio::time::sleep(duration).await;

                continue;
            }
        };

        server_stream.set_nodelay(true)?;
        client_stream.set_nodelay(true)?;

        let (server_read, server_write) = server_stream.into_split();
        let (client_read, client_write) = client_stream.into_split();

        println!(
            "Created channel: {} <---> {}",
            client_address, server_address,
        );

        let left_pipe = tokio::spawn(async move { copy_data(client_read, server_write).await });
        let right_pipe = tokio::spawn(async move { copy_data(server_read, client_write).await });

        let _ = left_pipe.await?;
        let _ = right_pipe.await?;

        info!("Channel closed");
    }
}

async fn client_server_connect(
    client_address: &str,
    server_address: &str,
    timeout: u64,
) -> Result<(), Box<dyn Error>> {
    let duration = Duration::from_secs(timeout);

    info!("Server server mode");
    info!("Client address {}", client_address);
    info!("Server address {}", server_address);

    let server_listener = TcpListener::bind(server_address).await?;

    loop {
        info!("Connecting to client {}...", client_address);

        let client_stream = match TcpStream::connect(client_address).await {
            Ok(stream) => stream,
            Err(error) => {
                warn!("Failed to connect: {}.", error);

                tokio::time::sleep(duration).await;

                continue;
            }
        };

        info!("Waiting connections...");

        let (server_stream, _address) = server_listener.accept().await?;

        client_stream.set_nodelay(true)?;
        server_stream.set_nodelay(true)?;

        let (client_read, client_write) = client_stream.into_split();
        let (server_read, server_write) = server_stream.into_split();

        println!(
            "Created channel: {} <---> {}",
            client_address, server_address,
        );

        let left_pipe = tokio::spawn(async move { copy_data(client_read, server_write).await });
        let right_pipe = tokio::spawn(async move { copy_data(server_read, client_write).await });

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
        Options::ClientServer {
            client_address,
            server_address,
            mode: ClientServerMode::Listen,
            timeout,
        } => client_server_listen(&client_address, &server_address, timeout).await,
        Options::ClientServer {
            client_address,
            server_address,
            mode: ClientServerMode::Connect,
            timeout,
        } => client_server_connect(&client_address, &server_address, timeout).await,
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
        Options::ClientClient {
            external_address,
            internal_address,
            timeout,
            mode: ClientClientMode::ConnectBoth,
        } => client_client_both(&external_address, &internal_address, timeout).await,
    }
}
