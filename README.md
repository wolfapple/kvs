# KVS (Key-Value Store)

This is a personal toy project for learning the Rust file API and storage engine design, inspired by the [PingCAP talent plan](https://github.com/pingcap/talent-plan).

It's a simple on-disk key/value store that can be modified and queried from the command line. It uses a simplification
of the storage algorithm from Bitcask, chosen for its combination of simplicity and effectiveness.

## Project Goals

This project is developed in several stages:

1. **Log-based Storage**: All write commands are sequentially written to a log on disk (a Write-Ahead Log). On startup,
   this log is read to restore the in-memory database state.
2. **In-memory Indexing**: To optimize memory usage, only the keys and their corresponding offsets (positions) in the
   disk log are stored in memory.
3. **Log Compaction**: To prevent the log from growing indefinitely, a log compaction feature is introduced to remove
   old or redundant data.

## Project Specification

The kvs project builds a library named `kvs` and a command-line tool that uses it, also named `kvs`.

### CLI

The `kvs` executable supports the following arguments:

* `kvs set <KEY> <VALUE>`
    * Sets the value of a string key. Prints an error and returns a non-zero exit code on failure.
* `kvs get <KEY>`
    * Gets the string value of a given key. Prints an error and returns a non-zero exit code on failure.
* `kvs rm <KEY>`
    * Removes a given key. Prints an error and returns a non-zero exit code on failure.
* `kvs -V`
    * Prints the version information.

### Library

The `kvs` library contains a `KvStore` type that supports the following methods:

* `KvStore::set(&mut self, key: String, value: String) -> Result<()>`
    * Sets the value of a string key. Returns an error if the value is not written successfully.
* `KvStore::get(&mut self, key: String) -> Result<Option<String>>`
    * Gets the string value of a key. Returns `None` if the key does not exist. Returns an error if the value is not
      read successfully.
* `KvStore::remove(&mut self, key: String) -> Result<()>`
    * Removes a given key. Returns an error if the key does not exist or is not removed successfully.
* `KvStore::open(path: impl Into<PathBuf>) -> Result<KvStore>`
    * Opens a `KvStore` at a given path. Returns a `KvStore` instance.