# amardiscord

A web UI for browsing Discord backups.

## Usage

`amardiscord` supports Discord udmps extracted via [this tool](https://github.com/StenniHub/discord-backup).

Once you have a dump, extract it into the `./data` directory in the root of this repository.

### Free-standing compilation

You can install `amardiscord` via Cargo:

```
# Compile the code
cargo install --locked --git https://github.com/soulsspeedruns/amardiscord

# Build the SQLite database from the dump
amardiscord build

# Serve the content
amardiscord serve
```

You can take the 

### Docker image

You can also run `amardiscord` as a Docker image. During the build process, the Discord dump
extracted in `./data` will be automatically built into a SQLite database and embedded in the
image, so you will be able to run it without requiring any other dependencies.

```
# Clone the repo
git clone https://github.com/soulsspeedruns/amardiscord && cd amardiscord

# Copy the backup in the `./data` directory
cp -a /path/to/discord/backup ./data

# Build and run the Docker image
docker build -t amardiscord .
docker run --rm -it -p 3000:3000 amardiscord
```
