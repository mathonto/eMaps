#!/usr/bin/env bash

sudo apt update
sudo apt install -y make curl

# node
curl -sL https://deb.nodesource.com/setup_12.x | sudo -E bash -
sudo apt-get install -y nodejs
# rust
curl https://sh.rustup.rs -sSf | sh -s -- -y
. ~/.cargo/env

make
