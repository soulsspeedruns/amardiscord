# amardiscord

A web UI for browsing Discord backups.

## Usage

`amardiscord` supports Discord backups extracted via [this tool](https://github.com/StenniHub/discord-backup).

The backup should have this tree structure:

```
$ ls --tree my-server-backup
 my-server-backup
├──  1168694611581865984.json
└──  1168694611581865984
    ├──  categories
    │   ├──  1.json
    │   ├──  10.json
    │   ├──  11.json
    │   ├──  12.json
    │   ├──  13.json
    │   ├──  14.json
    │   ├──  15.json
    │   ├──  2.json
    │   ├──  3.json
    │   ├──  8.json
    │   └──  9.json
    └──  other_channels
        └──  1.json
```

Note that the top-level `.json` file is ignored, and `other_channels` is optional.

### Free-standing compilation

You can install `amardiscord` via Cargo:

```
# Compile the code
cargo install --locked --git https://github.com/soulsspeedruns/amardiscord

# Build the SQLite database from the backup
amardiscord build /path/to/my-server-backup

# Serve the content
amardiscord serve
```

### Docker image

You can also deploy `amardiscord` as a Docker image. You will need to build your
`amardiscord.sqlite` database file via the CLI first.

Then, build and start the container, mounting the database file at `/app/amardiscord.sqlite`.

```
# Clone the repo
git clone https://github.com/soulsspeedruns/amardiscord && cd amardiscord

# Build the database file
cargo run --release -- build /path/to/my-server-backup

# Build and run the Docker image
docker build -t amardiscord .
docker run --rm -it \
    -p 3000:3000 \
    -v ./amardiscord.sqlite:/app/amardiscord.sqlite:ro \
    amardiscord
```
