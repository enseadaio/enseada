FROM golang:1.13-alpine as builder

ENV GO111MODULE=on
ENV CGO_ENABLED=0
ENV GOOS=linux
ENV GOARCH=amd64

WORKDIR /app

COPY go.mod .
COPY go.sum .

RUN go mod download

COPY . .

RUN go build -o bin/enseada ./cmd/enseada

# final stage
FROM scratch

ENV ENSEADA_ENV=production

COPY --from=builder /app/bin/enseada /app/enseada

EXPOSE 9623

ENTRYPOINT ["/app/enseada"]