# amardiscord

A web UI for browsing Discord backups.

## Usage

`amardiscord` supports Discord dumps extracted via [this tool](https://github.com/StenniHub/discord-backup).

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

### Docker image

You can also run `amardiscord` as a Docker image. During the build process, a Discord dump
will be retrieved from a HTTP URL, built into a SQLite database and embedded in the
image, so you will be able to run it without requiring any other dependencies.

Make sure to _always_ specify the `DATA_TARBALL_URL` build argument when building the image.

```
# Clone the repo
git clone https://github.com/soulsspeedruns/amardiscord && cd amardiscord

# Build and run the Docker image
docker build -t amardiscord . --build-arg DATA_TARBALL_URL=https://some.site/amardiscord-data.tar.gz
docker run --rm -it -p 3000:3000 amardiscord
```
