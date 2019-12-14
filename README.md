# Enseada
*A Cloud native multi-package registry*
  
[![version](https://img.shields.io/github/v/release/enseadaio/enseada?sort=semver)](https://github.com/enseadaio/enseada/releases/latest)
![build status](https://github.com/enseadaio/enseada/workflows/master/badge.svg)
[![docker pulls](https://img.shields.io/docker/pulls/enseada/enseada)](https://hub.docker.com/r/enseada/enseada)
[![license](https://img.shields.io/github/license/enseadaio/enseada)](./LICENSE)

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
If you have [Mage](https://magefile.org/) installed, you can build a new executable
from sources by simply running `mage` from the root folder and then running `bin/enseada`.

If you don't want to install Mage, that's perfectly fine.
Running `go run tools/mage.go` will use the vendored version of Mage and works exactly like the standalone executable.

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

Enseada will pick them up on its own. To embed them into the final executable use [go.rice](http://github.com/GeertJohan/go.rice):
```bash
# Install the CLI tool
go get github.com/GeertJohan/go.rice/rice

# Build the executable
go run tools/mage.go

# Embed the static assets
rice append --exec bin/enseada -i ./pkg/server

# Now bin/enseada is fully self-contained
```
## Local Database

A local CouchDB instance can be started using the provided [docker-compose.yml](./docker-compose.yml) file.
Simply run `docker-compose up -d` to start it in background, it will be available on `http://localhost:5984` and will
persist data in a Docker volume.

Upon first run, the database server is uninitialized. Please run the initialization setup for 
single node deployment by visiting http://localhost:5984/_utils/#setup and following the instructions.

## Configuration
Enseada is primarily configured via environment variables. Here is a list of the supported configuration.
Variables without a default value are required.

### Application config
```.env
## Application port
PORT=4000

## Logger level (accepts debug, info, warn, error)
LOG_LEVEL=info

## Base URL to use when serving packages 
## e.g. ASSET_HOST=https://d3gav2egqolk5.cloudfront.net
ASSET_HOST=nil
```

### Database

```.env
## CouchDB server URL
COUCHDB_URL

## CouchDB server username
COUCHDB_USER=nil

## CouchDB server password
COUCHDB_PASSWORD=nil
```

### Storage

```.env
# Storage provider (accepts gcs, s3, local) 
STORAGE_PROVIDER=local
```

#### Local
```.env
# Storage directory
STORAGE_DIR=uploads
```

#### S3
```.env
## S3 bucket name
AWS_S3_BUCKET

## S3 bucket region
AWS_REGION

## S3 client key ID (optional, defaults to instance role)
AWS_ACCESS_KEY_ID=nil

## S3 client secret key (optional, defaults to instance role)
AWS_SECRET_ACCESS_KEY=nil

## S3 HTTP endpoint (optional, defaults to Amazon S3 endpoints)
AWS_S3_ENDPOINT=nil

## Bucket keys prefix
BUCKET_PREFIX=uploads
```

#### GCS
```.env
## GCS bucket name
GCS_BUCKET

## GCS json credentials, alternative to GOOGLE_APPLICATION_CREDENTIALS
GCS_JSON_CREDENTIALS

## Path to a GCP credentials json, alternative to GCS_JSON_CREDENTIALS 
GOOGLE_APPLICATION_CREDENTIALS

## Bucket keys prefix
BUCKET_PREFIX=uploads
```


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

## Cluster mode
TBD

## License
This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.