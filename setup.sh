#!/bin/sh

set -e

wd="$PWD"
cd "$wd/api"
yarn install

cd "$wd/dashboard"
yarn install
yarn build

