# Enseada
*A Cloud native multi-package registry*
  
[![version](https://img.shields.io/github/v/release/enseadaio/enseada?sort=semver)](https://github.com/enseadaio/enseada/releases/latest)
![build status](https://github.com/enseadaio/enseada/workflows/master/badge.svg)
[![docker pulls](https://img.shields.io/docker/pulls/enseada/enseada)](https://hub.docker.com/r/enseada/enseada)
[![license](https://img.shields.io/github/license/enseadaio/enseada)](./LICENSE)
[![maintainability](https://api.codeclimate.com/v1/badges/c0bbc99aae02550fd5ad/maintainability)](https://codeclimate.com/github/enseadaio/enseada/maintainability)
[![test coverage](https://api.codeclimate.com/v1/badges/c0bbc99aae02550fd5ad/test_coverage)](https://codeclimate.com/github/enseadaio/enseada/test_coverage)

![logo](./.github/logo-white.png)


Enseada is a modern, fast and scalable package registry, designed from the ground up to run in elastic, container-based environments and to be highly available and distributed.

It leverages scalability by using natively distributed technologies

The registry itself is written in [Golang](https://golang.org/), a fast and resource efficient, statically compiled programming language
built for the Cloud.

[CouchDB](https://couchdb.apache.org/) is used as the primary datastore, containing information about
repositories, users and access control. CouchDB is a web-native database written in Erlang and based on web technologies
like HTTP and JSON.

As far as storage is concerned, both local disks an object storage services are supported, altough
the latter are strongly recommended for production deployments.

## Supported package repositories

Enseada is a multi-package registry, meaning it can support a large number of package 
formats and registry APIs.

At the moment, the following formats are supported:

- [Maven 2/3](https://maven.apache.org/guides/introduction/introduction-to-repositories.html)
- [NPM](https://github.com/npm/registry/blob/master/docs/REGISTRY-API.md) (planned, coming soon)
- [Docker](https://docs.docker.com/registry/spec/api/) (planned, coming soon)
- [RubyGems](https://rubygems.org) (planned, coming soon)

## Supported storage providers

The storage layer used by Enseada provides pluggable backends, allowing to easily support
multiple storage providers.
See [Configuration](#configuration) for how to setup the storage layer.

At the moment, only these providers are supported:

- Local disk
- S3 compatible (AWS S3, Minio, DigitalOcean Spaces, Scaleway Object Storage, Ceph, etc)
- Google Cloud Storage

Local disk is only supported in single-node mode. To support [cluster mode](#cluster-mode) use an object storage provider.

## Build

Enseada is built as a statically linked executable.
If you have [Make](https://www.gnu.org/software/make/) installed, you can build a new executable
from sources by simply running `make build-server` from the root folder and then running `bin/enseada-server`.

The following tasks are available:
```bash
$ make help

all                      Build standalone server binary (default)
build-server             Build server binary
build-client             Build client binary
test-bench               Run benchmarks
test-short               Run only short tests
test-verbose             Run tests in verbose mode with coverage reporting
test-race                Run tests with race detector
check test tests         Run tests
test-xml                 Run tests with xUnit output
test-coverage            Run coverage tests
lint                     Run golint
fmt                      Run gofmt on all source files
vet                      Run go vet on all source files
imports                  Run goimports on all source files
build-standalone-server  Build server binary with embedded static assets
web                      Build web assets with Webpack
wire                     Generate Wire code
proto                    Generate RPC code
deps                     Install dependencies
clean                    Cleanup everything
update-license           Update license headers
install-hooks            Install git hooks

```

Enseada is also packaged as a [Docker image](https://www.docker.com/). Build one with `docker build -t myname/enseada:latest .`

## Web UI

Enseada comes with a management web UI. To build it from sources, NodeJS and Yarn are required.

Execute the following commands to build the static assets.

```bash
# Go the the web directory
cd web

# Install all dependencies
yarn install

# Build for development (all stylesheets, no minification)
yarn build

# Build for production (minimal stylesheets, minification)
yarn build:prod
```

To build assets for production, you can also run `make web` that will do everything for you.

Enseada will pick them up on its own. To embed them into the final executable using [go.rice](http://github.com/GeertJohan/go.rice) run `make build-standalone-server`.

## Local Database

A local CouchDB instance can be started using the provided [docker-compose.yml](./docker-compose.yml) file.
Simply run `docker-compose up -d` to start it in background, it will be available on `http://localhost:5984` and will
persist data in a Docker volume.

Upon first run, the database server is uninitialized. Please run the initialization setup for 
single node deployment by visiting http://localhost:5984/_utils/#setup and following the instructions.

## HTTPS support
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

## License
This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.