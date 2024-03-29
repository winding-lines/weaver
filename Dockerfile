##################
# Multi stage build, first build the javascript assets
# We need node to build the web app
FROM node:11.6 as node

RUN USER=root npm install -g -s --no-progress yarn 
RUN mkdir /web
WORKDIR /web
COPY lib-server/web .
RUN /bin/sh build.sh


########################## BUILD IMAGE  ##########################
# We need to use the Rust build image, because
# we need the Rust compiler and Cargo tooling
FROM rust:1.31 as build
RUN apt-get update && apt-get install -y libdbus-1-dev libgmp-dev

#########

WORKDIR /app

# Copies the complete project
# To avoid copying unneeded files, use .dockerignore
COPY . .

# Get the web assets
COPY --from=node /web/dist lib-server/web/dist

# Build the rust project
RUN cargo build --release --all

######################## RUNTIME IMAGE  ########################
# Create a new stage with a minimal image
# because we already have a binary built
FROM debian:stretch-slim

# Install needed libraries
RUN apt-get update && apt-get install -y\
    openssl\
    ca-certificates\
    libdbus-1-dev libgmp-dev libsqlite3-dev\
    --no-install-recommends\
 && rm -rf /var/lib/apt/lists/*

RUN mkdir /data
VOLUME /data
EXPOSE 80
EXPOSE 3012

# Copies the files from the context (env file and web-vault)
# and the binary from the "build" stage to the current stage
COPY --from=build app/target/release/weaver-server .
COPY --from=build app/target/release/weaver-data .

# Configures the startup
CMD /weaver-data --password environment --location /data setup && /weaver-server \
  --password environment --location /data \
  --base-url /wr \
  start --fg --port 8080 --address 0.0.0.0
