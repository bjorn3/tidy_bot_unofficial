#!/bin/sh

echo $HEROKU_PRIVATE_KEY_CONTENT | tr "@" "\n" > $GITHUB_PRIVATE_KEY
cat $GITHUB_PRIVATE_KEY

export PATH="$HOME/.cargo/bin:$PATH"
./target/release/tidy_bot_unofficial
