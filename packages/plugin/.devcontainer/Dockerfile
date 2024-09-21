FROM --platform=linux/amd64 ghcr.io/0xcaff/ps4-sdk-images/goldhen:edge

# install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*
