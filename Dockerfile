# base image
FROM rust:1-buster as base

ARG NODE_VERSION=node_14.x
RUN curl -sSL https://deb.nodesource.com/gpgkey/nodesource.gpg.key | apt-key add -
RUN echo "deb https://deb.nodesource.com/$NODE_VERSION buster main" | tee /etc/apt/sources.list.d/nodesource.list
RUN echo "deb-src https://deb.nodesource.com/$NODE_VERSION buster main" | tee -a /etc/apt/sources.list.d/nodesource.list
RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add -
RUN echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list
RUN apt-get update -y && apt-get install build-essential llvm-dev libclang-dev nodejs yarn -y

# build server executable
FROM base as builder

# Dashboard
WORKDIR /app/enseada/dashboard

COPY dashboard/package.json .
COPY dashboard/yarn.lock .

RUN yarn --frozen-lockfile install
COPY dashboard .

# Workspace
WORKDIR /app/enseada
COPY Cargo.lock .
COPY Cargo.toml .

# CouchDB
WORKDIR /app/enseada/couchdb
COPY couchdb .

# Common Lib
WORKDIR /app/enseada/lib
COPY lib .

# Events
WORKDIR /app/enseada/events
COPY events .

# Maven
WORKDIR /app/enseada/maven
COPY maven .

# OAuth
WORKDIR /app/enseada/oauth
COPY oauth .

# Olly
WORKDIR /app/enseada/observability
COPY observability .

# OCI
WORKDIR /app/enseada/oci
COPY oci .

# RBAC
WORKDIR /app/enseada/rbac
COPY rbac .

# Users
WORKDIR /app/enseada/users
COPY users .

# API
WORKDIR /app/enseada/api
COPY api/package.json .
COPY api/yarn.lock .

RUN yarn --frozen-lockfile install

COPY api .

# Server
WORKDIR /app/enseada/server

COPY server/Cargo.toml .

RUN mkdir -p .cargo
RUN cargo vendor > .cargo/config

COPY server .

RUN cargo build --release

# final stage
FROM quay.io/bitnami/minideb:buster

RUN install_packages ca-certificates

RUN apt-get update && apt-get upgrade -y && \
    rm -r /var/lib/apt/lists /var/cache/apt/archives

WORKDIR /app/enseada

COPY --from=builder /app/enseada/target/release/enseada-server /app/enseada

EXPOSE 9623

ENTRYPOINT ["/app/enseada/enseada-server"]
