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

# Builds again, this time it'll just be
# your actual source files being built
RUN cargo build --release --all

######################## RUNTIME IMAGE  ########################
# Create a new stage with a minimal image
# because we already have a binary built
FROM debian:stretch-slim

# Install needed libraries
RUN apt-get update && apt-get install -y\
    openssl\
    ca-certificates\
    --no-install-recommends\
 && rm -rf /var/lib/apt/lists/*

RUN mkdir /data
VOLUME /data
EXPOSE 80
EXPOSE 3012

# Copies the files from the context (env file and web-vault)
# and the binary from the "build" stage to the current stage
COPY --from=build app/target/release/weaver-server .

# Configures the startup
CMD ./weaver-server --location /data start --fg --port 8080 
