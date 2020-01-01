MODULE   = $(shell env GO111MODULE=on $(GO) list -m)
DATE    ?= $(shell date +%FT%T%z)
VERSION ?= $(shell git describe --tags --always --dirty --match=v* 2> /dev/null || \
			cat $(CURDIR)/.version 2> /dev/null || echo v0)
PKGS     = $(or $(PKG),$(shell env GO111MODULE=on $(GO) list ./...))
TESTPKGS = $(shell env GO111MODULE=on $(GO) list -f \
			'{{ if or .TestGoFiles .XTestGoFiles }}{{ .ImportPath }}{{ end }}' \
			$(PKGS))
BIN      = $(CURDIR)/bin

GO      = go
TIMEOUT = 15
V = 0
Q = $(if $(filter 1,$V),,@)
M = $(shell printf "\033[34;1m▶\033[0m")

export GO111MODULE=on

.PHONY: all
all: fmt vet build-standalone-server ## Build standalone server binary (default)

# Build

.PHONY: build-server
build-server: proto | $(BIN); $(info $(M) building server executable…) @ ## Build server binary
	$Q $(GO) build \
		-tags release \
		-ldflags '-X $(MODULE)/cmd.Version=$(VERSION) -X $(MODULE)/cmd.BuildDate=$(DATE)' \
		-o $(BIN)/enseada-server ./cmd/enseada-server

.PHONY: build-client
build-client: proto | $(BIN); $(info $(M) building client executable…) @ ## Build client binary
	$Q $(GO) build \
		-tags release \
		-ldflags '-X $(MODULE)/cmd.Version=$(VERSION) -X $(MODULE)/cmd.BuildDate=$(DATE)' \
		-o $(BIN)/enseada ./cmd/enseada


# Tools

$(BIN):
	@mkdir -p $@
$(BIN)/%: | $(BIN) ; $(info $(M) building $(PACKAGE)…)
	$Q tmp=$$(mktemp -d); \
	   env GO111MODULE=off GOPATH=$$tmp GOBIN=$(BIN) $(GO) get $(PACKAGE) \
		|| ret=$$?; \
	   rm -rf $$tmp ; exit $$ret

GOLINT = $(BIN)/golint
$(BIN)/golint: PACKAGE=golang.org/x/lint/golint

GOIMPORTS = $(BIN)/goimports
$(BIN)/goimports: PACKAGE=golang.org/x/tools/cmd/goimports

GOCOV = $(BIN)/gocov
$(BIN)/gocov: PACKAGE=github.com/axw/gocov/...

GOCOVXML = $(BIN)/gocov-xml
$(BIN)/gocov-xml: PACKAGE=github.com/AlekSi/gocov-xml

GO2XUNIT = $(BIN)/go2xunit
$(BIN)/go2xunit: PACKAGE=github.com/tebeka/go2xunit

PROTOTOOL = $(BIN)/prototool
$(BIN)/prototool: PACKAGE=github.com/uber/prototool/cmd/prototool

ADDLICENSE = $(BIN)/addlicense
$(BIN)/addlicense: PACKAGE=github.com/google/addlicense

RICE = $(BIN)/rice
$(BIN)/rice: PACKAGE=github.com/GeertJohan/go.rice/rice

# Tests

TEST_TARGETS := test-default test-bench test-short test-verbose test-race
.PHONY: $(TEST_TARGETS) test-xml check test tests
test-bench:   ARGS=-run=__absolutelynothing__ -bench=. ## Run benchmarks
test-short:   ARGS=-short        ## Run only short tests
test-verbose: ARGS=-v            ## Run tests in verbose mode with coverage reporting
test-race:    ARGS=-race         ## Run tests with race detector
$(TEST_TARGETS): NAME=$(MAKECMDGOALS:test-%=%)
$(TEST_TARGETS): test
check test tests: fmt vet ; $(info $(M) running $(NAME:%=% )tests…) @ ## Run tests
	$Q $(GO) test -timeout $(TIMEOUT)s $(ARGS) $(TESTPKGS)

test-xml: fmt vet | $(GO2XUNIT) ; $(info $(M) running xUnit tests…) @ ## Run tests with xUnit output
	$Q mkdir -p test
	$Q 2>&1 $(GO) test -timeout $(TIMEOUT)s -v $(TESTPKGS) | tee test/tests.output
	$(GO2XUNIT) -fail -input test/tests.output -output test/tests.xml

COVERAGE_MODE    = atomic
COVERAGE_PROFILE = $(COVERAGE_DIR)/profile.out
COVERAGE_XML     = $(COVERAGE_DIR)/coverage.xml
COVERAGE_HTML    = $(COVERAGE_DIR)/index.html
.PHONY: test-coverage test-coverage-tools
test-coverage-tools: | $(GOCOV) $(GOCOVXML)
test-coverage: COVERAGE_DIR := $(CURDIR)/test/coverage.$(shell date -u +"%Y-%m-%dT%H:%M:%SZ")
test-coverage: fmt vet test-coverage-tools ; $(info $(M) running coverage tests…) @ ## Run coverage tests
	$Q mkdir -p $(COVERAGE_DIR)
	$Q $(GO) test \
		-coverpkg=$$($(GO) list -f '{{ join .Deps "\n" }}' $(TESTPKGS) | \
					grep '^$(MODULE)/' | \
					tr '\n' ',' | sed 's/,$$//') \
		-covermode=$(COVERAGE_MODE) \
		-coverprofile="$(COVERAGE_PROFILE)" $(TESTPKGS)
	$Q $(GO) tool cover -html=$(COVERAGE_PROFILE) -o $(COVERAGE_HTML)
	$Q $(GOCOV) convert $(COVERAGE_PROFILE) | $(GOCOVXML) > $(COVERAGE_XML)

.PHONY: lint
lint: | $(GOLINT) ; $(info $(M) running golint…) @ ## Run golint
	$Q $(GOLINT) -set_exit_status $(PKGS)

.PHONY: fmt
fmt: ; $(info $(M) running gofmt…) @ ## Run gofmt on all source files
	$Q $(GO) fmt $(PKGS)

.PHONY: vet
vet: ; $(info $(M) running go vet…) @ ## Run go vet on all source files
	$Q $(GO) vet $(PKGS)

.PHONY: imports
imports: | $(GOIMPORTS) ; $(info $(M) running goimports…) @ ## Run goimports on all source files
	$Q $(GOIMPORTS) -w ./cmd
	$Q $(GOIMPORTS) -w ./pkg
	$Q $(GOIMPORTS) -w ./internal
	$Q $(GOIMPORTS) -w ./rpc

.PHONY: build-standalone-server
build-standalone-server: build-server web | $(RICE) ; $(info $(M) building standalone server…) @ ## Build server binary with embedded static assets
	rice append --exec $(BIN)/enseada-server -i ./pkg/http -i ./pkg/auth

.PHONY: web
web: ; $(info $(M) build web assets…) @ ## Build web assets with Webpack
	cd web && yarn install && yarn build:prod

# Codegen

.PHONY: proto
proto: | $(PROTOTOOL) ; $(info $(M) generating RPC code…) @ ## Generate RPC code
	$(Q) $(PROTOTOOL) all ./rpc

# Misc

.PHONY: deps
deps: ; $(info $(M) installing dependencies…)	@ ## Install dependencies
	$(Q) $(GO) mod vendor

.PHONY: clean
clean: ; $(info $(M) cleaning…)	@ ## Cleanup everything
	@rm -rf $(BIN)
	@rm -rf test/tests.* test/coverage.*
	@rm .git/hooks/pre-commit
	@rm -rf web/static

.PHONY: help
help:
	@grep -E '^[ a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

.PHONY: version
version:
	@echo $(VERSION)

.PHONY: update-toc
update-toc:
	@markdown-toc -i --bullets="-" GUIDELINES.md

.PHONY: update-license
update-license: | $(ADDLICENSE) ; $(info $(M) updating license headers…) @ ## Update license headers
	$(Q) $(ADDLICENSE) -f ./rpc/copyright.txt ./cmd ./pkg ./internal ./rpc ./web

.PHONY: install-hooks
install-hooks: ; $(info $(M) installing git hooks…) @ ## Install git hooks
	@mkdir -p .git/hooks
	@cp ./githooks/pre-commit .git/hooks/pre-commit