#!/bin/bash

apt-get update && apt-get install -y curl
curl https://sh.rustup.rs -sSf | bash -s -- -y
. "$HOME/.cargo/env"
make -C packages/extern_traces_plugin
