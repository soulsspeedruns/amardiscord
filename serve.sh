#!/bin/bash

# TODO: Add argument for whether to run in production or debug mode
if [ -e "./data/amardiscord.sqlite" ]; then
    echo "SQLite database already exists, skipping build step."
else
    ./target/release/amardiscord build
fi

./target/release/amardiscord serve
