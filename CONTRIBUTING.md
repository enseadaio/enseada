# Contributing to Enseada
_Adapted from the [Atom Contribution Guide](https://github.com/atom/atom/blob/master/CONTRIBUTING.md)_

First off, thanks for taking the time to contribute!

The following is a set of guidelines for contributing to Enseada and its packages, which are hosted in the [Enseada Organization](https://github.com/enseadaio) on GitHub. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

<!-- toc -->

- [Code of Conduct](#code-of-conduct)
- [Getting started](#getting-started)
  * [Fork the repository](#fork-the-repository)
  * [Folder structure](#folder-structure)
  * [Local development](#local-development)
  * [Running tests](#running-tests)
  * [Linting](#linting)
  * [Pull Requests](#pull-requests)
  * [Sign your work](#sign-your-work)
- [Styleguides](#styleguides)
  * [Git Commit Messages](#git-commit-messages)
  * [Rust Code Styleguide](#rust-code-styleguide)

<!-- tocstop -->

## Code of Conduct

This project and everyone participating in it is governed by the [Enseada Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. <!-- Please report unacceptable behavior to [enseada@pm.me](mailto:enseada@pm.me). -->

## Getting started

### Fork the repository

If you are not a maintainer of the project you'll need to fork the codebase to your personal Github account.
Once done you can clone it locally by running `git clone git@github.com:$YOUR_ACCOUNT/enseada.git`.

### Folder structure

Enseada groups related functionalities in modules. Each module contains everything needed to implement
business logic except the HTTP controllers.
```
.
|-- api            # OpenAPI v3 definitions
|-- conformance    # Conformance tests for different package formats
|   `-- oci          # OCI v1 spec conformance test
|-- couchdb        # CouchDB client library
|-- dashboard      # Vue 2 application for the web GUI
|-- deploy         # Various deployment files and examples
|   |-- docker       # Deploy Enseada via docker-compose
|   `-- k8s          # Deploy Enseada on Kubernetes
|-- events         # Events module, provides the in-memory event bus library
|-- lib            # libenseada, generic common code used in many other modules
|-- maven          # Maven module, implements support for handling Maven packages
|-- oauth          # OAuth module, implements RFC6749 for providing OAuth 2.0 compliant endpoints
|-- observability  # Observability module, provides code for instrumenting and monitoring Enseada
|-- oci            # OCI module, implements the OpenContainers distribution spec v1 for handling container images
|-- rbac           # RBAC rule engine, provides authorization functionalities
|-- server         # Web server module, the main entrypoint for Enseada.
`-- users          # Users module, provides user management and authentication functionalities
```

### Local development

Enseada requires the latest stable Rust toolchain, as well as NodeJS and Yarn to build the web dashboard.  
If you use `rustup` it should have picked up the correct toolchain on its own. We also use `cargo-watch` to recompile
automatically when files change. Install it with `cargo install cargo-watch`.

To start the server you will need to have CouchDB 3 and Minio running. CouchDB is the distributed database used as the
primary datastore, while Minio provides an S3-compliant object storage used to store blobs and files.

If you use Docker, a handy `docker-compose.yml` file can be found in the root of the repository. The default configuration found
at [enseada.yml](./enseada.yml) is already prepared for using them this way. 

```bash
# In the repository root

# Start minio and couchdb
docker-compose up -d

# Start the server in watch mode (reloads code when changes occur, except in the dashboard directory
cargo watch -x run -i 'dashboard/'
```

Note that there is no need to manually install Rust or Node dependencies as this is automatically take care of by the
build process (see the [`api` module build script](old_api/build.rs) and [`server` module build script](old_api/build.rs)).

The server will be listening on `localhost:9623`. Navigating to this address via a browser will open the web dashboard.
The REST API documentation can be found at `http{s}://localhost:9623/api/docs`.

### Web dashboard

Enseada provides a web GUI for administration purposes. The dashboard is written in [VueJS 2](https://vuejs.org/) and can be
found in the `dashboard` directory. More information can be found in the related [README](./dashboard/README).

If you need to hack on the web dashboard too you can enable autoreload. Open a new terminal and run the following
commands:

```bash
cd dashboard

yarn watch
```

When changing files in the dashboard source code you only need to refresh the page to have it rendered from the server.

### Running tests

Tests can be run with the standard Rust test suite.

```bash
cargo test
```

### Linting

We use [Clippy] to format and lint the codebase. Install it by running `rustup component add clippy`.
Lint the codebase with a simple `cargo clippy`.

Note: we should strive to have no warnings from clippy on any part of the codebase, but it is not a hard requirement
for now. It will become imperative once we approach the first stable release.

### OpenAPI definitions

Enseada uses [BOATS](https://github.com/johndcarmichael/boats) to generate the server OpenAPI v3 spec.
The source code can be found in the `api` module. The [README](old_api/README.md) file contains more information
about the file structure and how to build the final spec. 
 
### Pull Requests

The process described here has several goals:

- Maintain Enseada's quality
- Fix problems that are important to users
- Engage the community in working toward the best possible Enseada
- Enable a sustainable system for Enseada's maintainers to review contributions

Please follow these steps to have your contribution considered by the maintainers:

1. Follow all instructions in [the template](.github/pull_request_template.md)
2. Follow the [styleguides](#styleguides)
3. After you submit your pull request, verify that all [status checks](https://help.github.com/articles/about-status-checks/) are passing <details><summary>What if the status checks are failing?</summary>If a status check is failing, and you believe that the failure is unrelated to your change, please leave a comment on the pull request explaining why you believe the failure is unrelated. A maintainer will re-run the status check for you. If we conclude that the failure was a false positive, then we will open an issue to track that problem with our status check suite.</details>

While the prerequisites above must be satisfied prior to having your pull request reviewed, the reviewer(s) may ask you to complete additional design work, tests, or other changes before your pull request can be ultimately accepted.

### Sign your work

The sign-off is a simple line at the end of the explanation for the patch. Your signature certifies that you wrote the patch or otherwise have the right to pass it on as an open-source patch. The rules are pretty simple: if you can certify the below (from developercertificate.org):

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.
1 Letterman Drive
Suite D4700
San Francisco, CA, 94129

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.


Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

Then you just add a line to every git commit message:

`Signed-off-by: Joe Smith <joe.smith@email.com>`

Use your real name (sorry, no pseudonyms or anonymous contributions.)

If you set your user.name and user.email git configs, you can sign your commit automatically with git commit -s.

## Styleguides

### Git Commit Messages

* Write your commit message in the imperative: "Fix bug" and not "Fixed
  bug" or "Fixes bug." This convention matches up with commit messages
  generated by commands like git merge and git revert.
* Limit the first line to 72 characters or less
* Reference issues and pull requests liberally after the first line
* When only changing documentation, include `[ci skip]` in the commit title
* Consider using the following message template:

```
[one line-summary of changes]

Because:
- [relevant context]
- [why you decided to change things]
- [reason you're doing it now]

This commit:
- [does X]
- [does Y]
- [does Z]

```

### Rust Code Styleguide

Enseada uses the linting rules provided by [Clippy](https://github.com/rust-lang/rust-clippy), the official Rust linter.

[beginner]:https://github.com/enseadaio/enseada/labels/good%20first%20issue
[help-wanted]:https://github.com/enseadaio/enseada/labels/help%20wanted
