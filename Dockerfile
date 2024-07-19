FROM rust:1.79-alpine3.20 AS chef
ENV RUSTFLAGS="-C target-feature=-crt-static"
ARG CARGO_TOKEN
RUN apk add --no-cache openssl-dev musl-dev
RUN cargo install cargo-chef
WORKDIR /usr/src/kon

FROM chef AS planner
COPY . .
RUN mkdir -p .cargo && \
  printf '[registries.gitea]\nindex = "sparse+https://git.toast-server.net/api/packages/toast/cargo/"\ntoken = "Bearer %s"\n' "$CARGO_TOKEN" >> .cargo/config.toml
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /usr/src/kon/recipe.json recipe.json
RUN cargo chef cook --release
COPY . .
RUN cargo build -rF production

FROM alpine:3.20
RUN apk add --no-cache libgcc
WORKDIR /kon
COPY --from=builder /usr/src/kon/target/release/kon .
CMD [ "./kon" ]
