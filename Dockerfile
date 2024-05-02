FROM rust:1.77-alpine3.19 AS chef
ENV RUSTFLAGS -C target-feature=-crt-static
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
RUN cargo build -r

FROM alpine:3.19@sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672bd4f27761f8e1ad6b
RUN apk add --no-cache libgcc
WORKDIR /kon
COPY --from=builder /usr/src/kon/target/release/kon .
COPY --from=builder /usr/src/kon/Cargo.toml .
CMD ./kon
