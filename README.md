# Simple Tunnel

Simple utility to create non encrypted connections between hosts. Utility can create tunnel only for two connections,
if single port used several time every next connection will be queued. In case if some connection closed - tunnel will
close all connections in pipe and start to wait next client.

NOTE: using this utility in public networks can lead to leaking your personal data, passwords or security keys.

Local database will be created in file `db.sqlite`. WEB server will be available on [localhost|http://localhost:8080/].

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
