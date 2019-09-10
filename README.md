# Prefix Server

This provides a lightweight service allowing clients to lookup tranasctions via prefix of the SHA256 digest of their inputs.

## Setting up Bitcoin

Bitcoin must be run with [RPC](https://bitcoin.org/en/developer-reference#remote-procedure-calls-rpcs) and raw transaction [ZMQ](https://github.com/bitcoin/bitcoin/blob/master/doc/zmq.md) enabled.

## Build

Install [Rust](https://www.rust-lang.org/tools/install) then

```bash
sudo apt install -y clang pkg-config libssl-dev libzmq3-dev
cargo build --release
```

The executable will be located at `./target/release/prefix-server`.

## Configuration

Settings may be given by `JSON`, `TOML`, `YAML`, `HJSON` and `INI` files and, by default, are located at `~/.prefix-server/config.*`.

| Name | Description | Default |
| - | - | - |
| `bind` | Bind address | `127.0.0.1:8080` |
| `node_ip` | Bitcoin IP | `127.0.0.1` |
| `rpc_port` | Bitcoin RPC port | `18443` |
| `rpc_username` | Bitcoin RPC username | `username` |
| `rpc_password` | Bitcoin RPC password | `password` |
| `zmq_port` | Bitcoin ZMQ port | `28332` |
| `db_path` | Database path | `~/.prefix-server/db` |
| `network` | Bitcoin network | `regnet` |

The `network` parameter must be either `mainnet`, `testnet` or `regnet`.

Each of the parameters above can be overloaded via command line (replacing `_` with `-`). Additionaly, `--config` can be passed via command line to specify a configuration file at a custom location.

A full list of command line arguments can be viewed via `prefix-server --help`.

## Running

```bash
./target/release/prefix-server [OPTIONS]
```
