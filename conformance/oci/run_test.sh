#!/usr/bin/env bash

set -e

# Registry details
export OCI_ROOT_URL="http://containers.localhost:9623"
export OCI_NAMESPACE="conformance/test"
export OCI_USERNAME="x-oauth-token"

# Generate and fill in
#
# The associated user must have the following permissions over the `conformance/test` repo:
# - image:pull
# - image:push
# - image:delete
#
# The token must have the following scopes:
# - oci:image:pull
# - oci:image:push
# - oci:image:delete
export OCI_PASSWORD="$ENSEADA_OCI_TEST_TOKEN"

# Which workflows to run
export OCI_TEST_PULL=1
export OCI_TEST_PUSH=1
export OCI_TEST_CONTENT_DISCOVERY=1
export OCI_TEST_CONTENT_MANAGEMENT=1

script=$(realpath "$0")
dir=$(dirname "$script")

if [[ ! -f "$dir/conformance.test" ]]; then
  echo "Test binary not found. Downloading.."
  go get -u github.com/opencontainers/distribution-spec/conformance
  cd "$(go env GOPATH)"/src/github.com/opencontainers/distribution-spec/conformance
  go test -c
  cp ./conformance.test "$dir"
  echo "Done"
fi

cd "$dir"
echo "Starting OCI Conformance Test"
./conformance.test
