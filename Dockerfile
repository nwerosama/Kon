FROM rust:1.77-alpine3.19@sha256:4a7925c3974fab3fb68b3f4d93d1a4a36cb201f9f87b01014def219b125f1960 AS compiler
ENV RUSTFLAGS="-C target-feature=-crt-static"
ARG CARGO_TOKEN
RUN apk add --no-cache openssl-dev musl-dev 
WORKDIR /usr/src/kon
COPY . .
RUN mkdir -p .cargo && \
  printf '[registries.gitea]\nindex = "sparse+https://git.toast-server.net/api/packages/toast/cargo/"\ntoken = "Bearer %s"\n' "$CARGO_TOKEN" >> .cargo/config.toml
RUN cargo fetch && cargo build -r

FROM alpine:3.19@sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672bd4f27761f8e1ad6b
RUN apk add --no-cache openssl-dev libgcc
WORKDIR /kon
COPY --from=compiler /usr/src/kon/target/release/kon .
COPY --from=compiler /usr/src/kon/Cargo.toml .
CMD [ "./kon" ]
