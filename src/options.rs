use std::str::FromStr;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "simple-tunnel",
    about = "Create simple not encrypted tunnels between hosts."
)]
pub enum Options {
    ServerServer {
        #[structopt(short = "e", long = "external")]
        external_address: String,

        #[structopt(short = "i", long = "internal")]
        internal_address: String,
    },
    ClientServer {
        #[structopt(short = "c", long = "client")]
        client_address: String,

        #[structopt(short = "s", long = "server")]
        server_address: String,

        #[structopt(short = "m", long = "mode")]
        mode: ClientServerMode,

        #[structopt(short = "t", long = "timeout", default_value = "10")]
        timeout: u64,
    },
    ClientClient {
        #[structopt(short = "e", long = "external")]
        external_address: String,

        #[structopt(short = "i", long = "internal")]
        internal_address: String,

        #[structopt(short = "m", long = "mode")]
        mode: ClientClientMode,

        #[structopt(short = "t", long = "timeout", default_value = "10")]
        timeout: u64,
    },
}

#[derive(Debug)]
pub enum ClientServerMode {
    Listen,
    Connect,
}

impl FromStr for ClientServerMode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "listen" => Ok(ClientServerMode::Listen),
            "connect" => Ok(ClientServerMode::Connect),
            _ => Err("Invalid mode, expected one of listen or connect"),
        }
    }
}

#[derive(Debug)]
pub enum ClientClientMode {
    ConnectExternal,
    ConnectInternal,
    ConnectBoth,
}

impl FromStr for ClientClientMode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "external" => Ok(ClientClientMode::ConnectExternal),
            "internal" => Ok(ClientClientMode::ConnectInternal),
            "both" => Ok(ClientClientMode::ConnectBoth),
            _ => Err("Invalid mode, expected one of external, internal or both"),
        }
    }
}
