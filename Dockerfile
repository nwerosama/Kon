FROM rust:1.80-alpine3.20@sha256:596c7fa13f7458097b8c88ad83f33420da0341e2f5b544e34d9aa18a22fe11d0 AS chef
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache openssl-dev musl-dev
RUN cargo install cargo-chef
WORKDIR /usr/src/kon

FROM chef AS planner
COPY . .
RUN cargo chef prepare

FROM chef AS dependencies
COPY --from=planner /usr/src/kon/recipe.json recipe.json
RUN cargo chef cook --release

FROM chef AS builder
COPY --from=planner /usr/src/kon/.cargo /usr/src/kon/.cargo
COPY --from=dependencies /usr/src/kon/target /usr/src/kon/target
COPY . .
RUN cargo build -rF production

FROM alpine:3.20@sha256:0a4eaa0eecf5f8c050e5bba433f58c052be7587ee8af3e8b3910ef9ab5fbe9f5
RUN apk add --no-cache libgcc fluidsynth
WORKDIR /kon
COPY --from=builder /usr/src/kon/target/release/kon .
CMD [ "./kon" ]
