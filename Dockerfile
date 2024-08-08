FROM rust:1.80-alpine3.20 AS chef
ENV RUSTFLAGS="-C target-feature=-crt-static"
ARG GIT_HASH
ENV GIT_COMMIT_HASH=${GIT_HASH}
RUN apk add --no-cache openssl-dev musl-dev
RUN cargo install cargo-chef
WORKDIR /builder

FROM chef AS planner
COPY . .
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /builder/recipe.json recipe.json
RUN cargo chef cook --release
COPY . .
RUN cargo build --offline -rF production

FROM alpine:edge
LABEL org.opencontainers.image.source="https://git.toast-server.net/toast/Kon"
RUN apk add --no-cache libgcc fluidsynth
WORKDIR /kon
COPY --from=builder /builder/target/release/kon .
CMD [ "./kon" ]
