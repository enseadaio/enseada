NAME := "enseada"
REPO := "enseadaio"
VERSION := `git rev-parse HEAD`
SEMVER_VERSION := `grep version server/Cargo.toml | awk -F"\"" '{print $2}' | head -n 1`
DOCKER_CMD := "docker"

set dotenv-load := false

default:
  @just --list --unsorted | grep -v "    default"

run:
  cargo run --bin enseada-server

watch:
  cargo-watch --clear --exec 'run --bin enseada-server'

build *args: fmt
  cargo build {{args}}

test:
  cargo test

fmt: _fmt _clippy

_fmt:
  cargo fmt

_clippy:
  cargo clippy --fix --allow-dirty --allow-staged
