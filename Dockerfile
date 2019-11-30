FROM elixir:1.9-alpine as builder

WORKDIR /app

# Set environment variables for building the application
ENV MIX_ENV=prod \
    LANG=C.UTF-8

# Install system dependencies
RUN apk update && apk add build-base

# Install hex and rebar
RUN mix local.hex --force && \
    mix local.rebar --force

COPY mix.exs .
COPY mix.lock .

RUN mix deps.get
RUN mix deps.compile

COPY config ./config
COPY lib ./lib
COPY priv ./priv
COPY assets ./assets

RUN mix phx.digest

RUN mix release

FROM alpine:3.9

ENV LANG=C.UTF-8
ENV STORAGE_DIR=/var/lib/enseada/data

RUN apk update && apk add ncurses-libs libstdc++

RUN adduser -D app

RUN mkdir -p $STORAGE_DIR
RUN chown -R app: $STORAGE_DIR

WORKDIR /home/app

COPY --from=builder /app/_build/prod/rel/enseada .

RUN chown -R app: .

USER app

CMD ["bin/enseada", "start"]
