#!/bin/bash

apt-get update && apt-get install -y curl
curl https://sh.rustup.rs -sSf | bash -s -- -y
export PATH="/root/.cargo/bin:${PATH}"
make -C packages/extern_traces_plugin
