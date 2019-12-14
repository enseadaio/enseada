FROM node:12-alpine as assets

ENV NODE_ENV=production

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

COPY --from=assets /web/static ./web

RUN go build -o bin/enseada ./cmd/enseada
RUN rice append --exec bin/enseada -i ./pkg/server

# final stage
FROM scratch

ENV ENSEADA_ENV=production

COPY --from=builder /app/bin/enseada /app/enseada

EXPOSE 9623

ENTRYPOINT ["/app/enseada"]