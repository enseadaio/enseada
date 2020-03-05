# Enseada
*A Cloud native multi-package registry*
  
[![version](https://img.shields.io/github/v/release/enseadaio/enseada?sort=semver)](https://github.com/enseadaio/enseada/releases/latest)
![build status](https://github.com/enseadaio/enseada/workflows/master/badge.svg)
[![docker pulls](https://img.shields.io/docker/pulls/enseada/enseada)](https://hub.docker.com/r/enseada/enseada)
[![license](https://img.shields.io/github/license/enseadaio/enseada)](./LICENSE)
[![Discord Badge](https://discordapp.com/api/guilds/667303788532465665/widget.png?style=shield)](https://discord.gg/A34Qt8A)

![logo](./.github/logo-white.png)

>[!WARNING]
>Enseada is still under initial development. Some of the features and characteristics
>described in this document may still be missing. 

Enseada is a modern, fast and scalable package registry, designed from the ground up to run in elastic, container-based environments and to be highly available and distributed.

It leverages scalability by using natively distributed technologies.

Check out the [official documentation](https://docs.enseada.io) for a complete manual of operation.

## Features

- [Multiple package repositories](#supported-package-repositories)
- [Multiple storage backends](#supported-storage-providers)
- Strong authentication based on [OAuth 2.0](https://auth0.com/docs/protocols/oauth2) tokens
- Flexible ACL engine to manage user permissions 
- Complete [management API](https://docs.enseada.io/developers/apis.html)
- CDN and caching friendly

The registry itself is written in [Rust](https://rust-lang.org/), a fast, resource efficient and statically compiled programming language
built for safety and speed.

[CouchDB](https://couchdb.apache.org/) is used as the primary datastore, containing information about
repositories, users and access control. CouchDB is a web-native database written in Erlang and based on web technologies
like HTTP and JSON.

Enseada stores packages in distributed and fault-tolerant data storage services. By default it stores them as file attachments
in CouchDB, so no configuration is needed. This works well if the number and size of packages is fairly low.
Cloud object storage services are also supported and are recommended for production use.


## Supported package repositories

Enseada is a multi-package registry, meaning it can support a large number of package
formats and registry APIs.

At the moment, the following formats are supported:

- [Docker](https://docs.docker.com/registry/spec/api/)
- [Maven 2/3](https://maven.apache.org/guides/introduction/introduction-to-repositories.html)
- [NPM](https://github.com/npm/registry/blob/master/docs/REGISTRY-API.md) (planned, coming soon)
- [RubyGems](https://rubygems.org) (planned, coming soon)
- [Rust crates](https://doc.rust-lang.org/cargo/reference/registries.html) (planned, coming soon)
- [Go module proxy](https://docs.gomods.io/intro/protocol/) (planned, coming soon)

## Supported storage providers

The storage engine used by Enseada provides pluggable backends, allowing to easily support
multiple storage providers.
See the [configuration guide](https://docs.enseada.io/users/configuration.html) for how to setup the storage layer.

At the moment, only these providers are supported:

- S3 compatible (AWS S3, Minio, DigitalOcean Spaces, Scaleway Object Storage, Ceph, etc)
- Google Cloud Storage
- Microsoft Azure Blobs
- CouchDB

The CouchDB provider allows to store packages as [file attachments](https://docs.couchdb.org/en/stable/api/document/attachments.html) in CouchDB.
This has the advantage of working out of the box and without requiring a third party service to store packages. However performance can become a problem
if the number of stored packages is very high, or if files are very large. It also means that properly backing up and replicating the CouchDB cluster
becomes of high importance. For real production workloads it is recommended to use a cloud object storage service instead. 

## HTTPS and HTTP/2 support
Enseada has full support for strict HTTPS, enabling it is very simple.

Passing the environment value `SSL=yes|true|active` (or any kind of non-empty value) will turn on
HTTPS on the entire application (with [HSTS](https://en.wikipedia.org/wiki/HTTP_Strict_Transport_Security) enabled). This will require two
additional environment variables.

```.env
## The path to the key file
SSL_KEY_PATH=nil

## The path to the certificate file
SSL_CERT_PATH=nil
```

When HTTPS is active, Enseada switches automatically to [HTTP/2](https://en.wikipedia.org/wiki/HTTP/2) if supported
by the client.

## Build

Enseada is built as a statically linked executable. The only dynamic dependencies are OpenSSL and libc.
However it needs a few native libraries to build the executable, in particular libclang, libssl and LLVM. On Debian
they can be installed with `apt install build-essential libssl-dev llvm-dev libclang-dev`.
 
You need a nightly build of the Rust compiler and standard library to build the binary. You can install one
using [rustup](https://rustup.rs).

Standard `cargo` commands work perfectly fine. For example, `cargo run` will compile and start the server.

Enseada is also packaged as a [Docker image](https://www.docker.com/r/enseada/enseada). Build one with `docker build -t myname/enseada:latest .`

## Web UI

Enseada comes with a management web UI. To build it from sources, NodeJS and Yarn are required.

Execute the following commands to build the static assets.

```bash
# Install all dependencies
yarn install

# Build and recompile on change for development (all stylesheets, no minification)
yarn watch

# Build for production (minimal stylesheets, minification, sourcemaps)
yarn build
```

Enseada will pick them up on its own.

## Local Database

A local CouchDB instance can be started using the provided [docker-compose.yml](./docker-compose.yml) file.
Simply run `docker-compose up -d` to start it in background, it will be available on `http://localhost:5984` and will
persist data in a Docker volume.

Upon first run, the database server is uninitialized. Please run the initialization setup for 
single node deployment by visiting http://localhost:5984/_utils/#setup and following the instructions.

## Local Minio Server

A local [Minio](https://minio.io) server can be started using the provided [docker-compose.yml](./docker-compose.yml) file.
Simply run `docker-compose up -d` to start it in background, it will be available on `http://localhost:9000` and will
persist data in a Docker volume.

Upon first run no bucket is present. Create a new one and configure the name in the appropriate environment variable 
in the Enseada configuration.

## Security Policy

If you want to report a security vulnerability, please follow the steps which we have defined for you in our [security policy](https://github.com/enseadaio/enseada/security/policy).

## Chat

Need some help or want to have a chat? Join our [Discord server](https://discord.gg/A34Qt8A)!  
![Discord Banner](https://discordapp.com/api/guilds/667303788532465665/widget.png?style=banner2)

## License
This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.
