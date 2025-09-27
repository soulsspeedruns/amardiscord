# amardiscord

A tool for reading Discord backups.

## Usage

First, you need to create a SQLite database from the JSON dump.
Extract the dump into `./data` and then run:

```
./amardiscord build
```

Then, you can serve the content:

```
./amardiscord serve
```

## Build

```
cargo build --release
```
