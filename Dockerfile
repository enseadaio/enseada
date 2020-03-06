# build frontend assets
FROM node:12 as assets

WORKDIR /web

COPY package.json .
COPY yarn.lock .

RUN yarn install

COPY static static

RUN yarn build

# build server executable
FROM rust:1-buster as builder

RUN apt-get update -y && apt-get install build-essential llvm-dev libclang-dev -y

WORKDIR /app/enseada

COPY Cargo.lock .
COPY Cargo.toml .

RUN mkdir .cargo
RUN cargo vendor > .cargo/config

COPY . .

RUN cargo build --release

# final stage
FROM bitnami/minideb:buster

RUN install_packages ca-certificates libc6

RUN apt-get update && apt-get upgrade -y && \
    rm -r /var/lib/apt/lists /var/cache/apt/archives
WORKDIR /app/enseada

COPY --from=builder /app/enseada/target/release/enseada-server /app/enseada
COPY --from=assets /web/dist /app/enseada/dist

EXPOSE 9623

ENTRYPOINT ["/app/enseada/enseada-server"]