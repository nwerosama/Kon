FROM rust:1.75-alpine3.19@sha256:2e505c3e2863b0a4627219ccd538aeef3de5f5907046f3f59ce9c1b6150d97ef AS compiler
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache openssl-dev musl-dev 
WORKDIR /usr/src/kon
COPY . .
RUN cargo fetch && cargo build -r

FROM alpine:3.19@sha256:b5dfcf7427d7ba35bd7eb083754c14008e50fc80b00a42b55946ccbc2cc6322a
RUN apk add --no-cache openssl-dev libgcc
WORKDIR /kon
COPY --from=compiler /usr/src/kon/target/release/kon .
COPY --from=compiler /usr/src/kon/Cargo.toml .
CMD [ "./kon" ]
