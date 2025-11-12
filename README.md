# amardiscord

A web UI for browsing Discord backups.

## Usage

`amardiscord` supports Discord backups extracted via [this tool](https://github.com/StenniHub/discord-backup).

The backup should have this tree structure:

```
$ ls --tree data
 data
└──  my_discord_backup
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

Note that any top-level `.json` files are ignored, and `other_channels` is optional.

### Free-standing deployment

You can install `amardiscord` via Cargo:

```
# Compile the code
cargo install --locked --git https://github.com/soulsspeedruns/amardiscord

# Build the SQLite cache database and serve the content (path is optional, defaults to `./data`).
amardiscord serve /path/to/backup
```

### Docker image

You can also deploy `amardiscord` as a Docker image, mounting your Discord backup at `/app/data` in read-write mode.

```
# Clone the repo
git clone https://github.com/soulsspeedruns/amardiscord && cd amardiscord

# Build the Docker image
docker build -t amardiscord .

# Run the Docker container and mount the data directory containing your Discord backup
docker run --rm -it \
    -p 3000:3000 \
    -v ./data:/app/data \
    amardiscord
```

#### Remount the backup directory in read-only mode

On the first run, a SQLite cache database named `amardiscord.sqlite` is created in the `/app/data` directory of the container. This is why it is necessary to have the bind mount in read-write mode at first.

On successive runs, `amardiscord` won't write anything else to the filesystem, so the directory can be freely mounted in read-only mode.
