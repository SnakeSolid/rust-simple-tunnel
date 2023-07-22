# Simple Tunnel

Simple utility to create non encrypted connections between hosts. Utility can create tunnel only for two connections,
if single port used several time every next connection will be queued. In case if some connection closed - tunnel will
close all connections in pipe and start to wait next client.

NOTE: using this utility in public networks can lead to leaking your personal data, passwords or security keys.

## Command line options

Utility support several model as subcommands:

### server-server

In this mode two listening sockets will be created. When clients connected to both sockets utility create pipe between
clients. This subcommand support following options:

* `-e`, `--external` <address:port> - external address, example `192.168.1.1:1080`;
* `-i`, `--internal` <address:port> - internal address, example `127.0.0.1:1080`;

Both addresses are required, and must be different. Addresses have the same processing rule, names show only pipe side.

### client-client

In this mode two listening sockets will be created. When clients connected to both sockets utility create pipe between
clients. This subcommand support following options:

* `-e`, `--external` <address:port> - external address, example `192.168.1.1:1080`;
* `-i`, `--internal` <address:port> - internal address, example `127.0.0.1:1080`;
* `-m`, `--mode` <mode> - connection mode, required parameter (one of `external`, `internal` or `both`);
* `-t`, `--timeout` <timeout> - interval between reconnections in seconds, default: 10;
t
Connection modes can be used to control which client will be connected first:

* `external` - connect to external server and wait for first packet, then connect to internal server;
* `internal` - connect to internal server and wait for first packet, then connect to external server;
* `both` - connect to both server simultaneously.

Modes `external` and `internal` can be used if server automatically close inactive connections.

## Example

We want to work with a notebook using an RDP connection from our computer. However, the firewall blocks all incoming
connections from `192.168.1.0/24` network. But we can still connect to any resource in `192.168.1.0/24` from notebook,
see image below.

![User Interface](images/example.svg "Computer in example network")

The solution is to create a TCP tunnel between the address `127.0.0.1:3389` on the notebook and the address
`127.0.0.1:3389` on the computer. Using two instances of `simple-tunnel` to create a pipe with on-demand connecting
and automatic restoring, using the following commands:

```sh
# For notebook
./simple-tunnel client-client --external 192.168.1.2:3389 --internal 127.0.0.1:3389 --mode external
```

This command will start `simple-tunnel` in client to client mode. It will attempt a connection to `192.168.1.2:3389`,
and upon receipt of the first data packet from the receives the first packet of data from the connection, it will
connect to the RDP server at `127.0.0.1:3389` and create a channel.

```sh
# For computer
./simple-tunnel server-server --external 192.168.1.2:3389 --internal 127.0.0.1:3389
```

This command starts `simple-tunnel` in server to server mode. It will listen on both addresses: `192.168.1.2:3389` and
`127.0.0.1:3389`. When both clients are connected, the utility will create a channel between notebook and computer
ports. RDP server will be available at computer address `127.0.0.1:3389`.

NOTE: formally, the tunnel from this example can be created using SSH port forwarding if available.

## Build

To build `roi-parser` from source code use following command:

```sh
cargo build --release
```

To start `simple-tunnel` use following command:

```sh
./target/release/simple-tunnel --help
```

## License
[license]: #license

Source code is primarily distributed under the terms of the MIT license. See LICENSE for details.
