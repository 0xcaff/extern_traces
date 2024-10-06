#!/bin/bash

curl https://sh.rustup.rs -sSf | bash -s -- -y
make -C packages/extern_traces_plugin
