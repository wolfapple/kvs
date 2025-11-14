# KVS (Key-Value Store)

This is a personal toy project for learning the Rust file API and storage engine design, inspired by the [PingCAP talent plan](https://github.com/pingcap/talent-plan).

It's a simple on-disk key/value store that supports multiple storage engines and can be accessed over the network.

## Project Goals

This project is developed in several stages:

1.  **Log-based Storage**: All write commands are sequentially written to a log on disk (a Write-Ahead Log). On startup, this log is read to restore the in-memory database state.
2.  **In-memory Indexing**: To optimize memory usage, only the keys and their corresponding offsets (positions) in the disk log are stored in memory.
3.  **Log Compaction**: To prevent the log from growing indefinitely, a log compaction feature is introduced to remove old or redundant data.
4.  **Client/Server Architecture**: The key-value store is exposed through a server, and a separate client can be used to interact with it.
5.  **Pluggable Storage Engines**: The server can be configured to use different storage engines. This project provides two engines:
    *   `kvs`: The original log-structured file-based storage engine.
    *   `sled`: An engine based on the `sled` embedded database.
6.  **Concurrent Server**: The server is improved to handle requests from multiple clients concurrently using a thread pool. Each connection is handled in a separate thread.

## Project Specification

The kvs project builds a library named `kvs` and two command-line tools: `kvs-server` and `kvs-client`.

### Server (`kvs-server`)

The `kvs-server` executable starts the key-value store server.

*   `kvs-server [--addr IP:PORT] [--engine ENGINE-NAME]`
    *   `--addr <IP:PORT>`: Sets the server address and port. Defaults to `127.0.0.1:4000`.
    *   `--engine <ENGINE-NAME>`: Sets the storage engine. Can be `kvs` or `sled`. If not specified, it will use the engine that was used last time in the current directory, or `kvs` if it's the first time.
*   `kvs-server -V`
    *   Prints the version information.

### Client (`kvs-client`)

The `kvs-client` executable is the command-line client to interact with the server.

*   `kvs-client set <KEY> <VALUE> [--addr IP:PORT]`
    *   Sets the value of a string key.
    *   `--addr <IP:PORT>`: Specifies the server address to connect to. Defaults to `127.0.0.1:4000`.
*   `kvs-client get <KEY> [--addr IP:PORT]`
    *   Gets the string value of a given key.
*   `kvs-client rm <KEY> [--addr IP:PORT]`
    *   Removes a given key.
*   `kvs-client -V`
    *   Prints the version information.

### Library

The `kvs` library provides the building blocks for the key-value store.

*   `KvsEngine` trait: An interface for a key-value storage engine, designed to be safely shared across multiple threads.
*   `KvStore`: A log-structured storage engine implementing the `KvsEngine` trait.
*   `SledKvsEngine`: A `sled`-based storage engine implementing the `KvsEngine` trait.
*   `KvsServer`: A server that can run with any type that implements `KvsEngine`.
*   `KvsClient`: A client for communicating with the `KvsServer`.
*   `ThreadPool` trait: An interface for the server's concurrency model.
