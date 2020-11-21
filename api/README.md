# Enseada OpenAPI V3 specification

This module contains [BOATS](https://github.com/johndcarmichael/boats) templates
that will make up the final YAML file with the OpenAPI definitions.

## Quickstart

This project needs NodeJS and Yarn installed.
Install the build dependencies by running `yarn install`.

The [docs](./docs) folder contains Nunjucks templates, structured as per BOATS requirements.
Head over to its [documentation](https://johndcarmichael.github.io/boats) for a primer on how it works.

The [src](./src) folder contains Rust structures that map to OpenAPI components. They are hand-written for now, but eventually
they will be generated from the OpenAPI specification.

#### Build a local spec

To test the specification build it locally by running `yarn build:local`. It will produce a [`dist/openapi.yml`](./dist/openapi.yml) with the full spec.

To render it run `yarn serve` and point a browser to the provided URL.
