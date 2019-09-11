# Prefix Server
[![Build Status](https://travis-ci.org/hlb8122/prefix-server.svg?branch=master)](https://travis-ci.org/hlb8122/prefix-server)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

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
| `min_prefix` | Minimum prefix length (in bytes) | `2` |

Each of the parameters above can be overloaded via command line (replacing `_` with `-`). Additionaly, `--config` can be passed via command line to specify a configuration file at a custom location.

A full list of command line arguments can be viewed via `prefix-server --help`.

## Running

```bash
./target/release/prefix-server [OPTIONS]
```

## Usage

The server has three endpoints:

### Prefix Search

The endpoint `/prefix/{prefix}` allows you to search indexed inputs by prefix. 

One may include a query string `/prefix/{prefix}?start={start}&end={end}` to filter the search.

The returned value is of the following form:

```javascript
{
    "results": [
        {
            "raw_tx": "02000000019b69251560ea1143de610...", // raw transaction
            "input_index": 3, // index of matching input
            "time": 1568170483 // transaction time
        },
        {
            "raw_tx": "02000000019b69251560ea1143de610...", // raw transaction
            "input_index": 1, // index of matching input
            "time": 1568170214 // transaction time
        }
    ]
}
```

The errors are as follows:

| Status Code | Body | Description |
| - | - | - |
| 404 | prefix not found | The prefix did not match any indexed items |
| 400 | invalid hex | The prefix was not in hexidecimal format |
| 500 | client error | There was an error communicating with bitcoind |

### Scrape (WIP)

The endpoint `/scrape` taking the following JSON

```javascript
{
    "start": 594274, // start height
    "end": 594279 // end height
}
```

allows the indexing of all transactions between and including two block numbers.

While the scraping is in progress the servers status will change from "idle" to "scraping" and calling `/scrape` again during this time will raise an error.

The errors are as follows:

| Status Code | Body | Description |
| - | - | - |
| 400 | invalid json | The JSON didn't meet the format above |
| 400 | empty interval | The interval was empty |
| 500 | scrape in progress | An scrape is already in progress |

### Status (WIP)

The endpoint `/status` returns the current state of the prefix server. This will either return `idle` or `scraping(594274, 594276, 594279)` where the tuple consists of the start, current and end block numbers of the scrape.
