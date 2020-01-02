FROM node:12-alpine as assets

WORKDIR /web

COPY web/package.json .
COPY web/yarn.lock .

RUN yarn install

COPY web .

RUN yarn build:prod

FROM golang:1.13-alpine as builder

ENV GO111MODULE=on
ENV CGO_ENABLED=0
ENV GOOS=linux
ENV GOARCH=amd64

WORKDIR /app

RUN go get github.com/GeertJohan/go.rice/rice

COPY go.mod .
COPY go.sum .

RUN go mod download

COPY . .

COPY --from=assets /web/static ./web/static

RUN go build -o bin/enseada-server ./cmd/enseada-server
RUN rice append --exec bin/enseada-server -i ./pkg/http -i ./pkg/auth

# final stage
FROM scratch

ENV STORAGE_DIR=/var/lib/enseada/data

COPY --from=builder /app/bin/enseada-server /app/enseada-server

EXPOSE 9623

ENTRYPOINT ["/app/enseada-server"]