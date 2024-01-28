FROM rust:1.75-alpine3.19@sha256:65aa0b28d02612a3811a7fd0c65b56e4ba766c35cef71965f1cacae7555771a0 AS compiler
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache openssl-dev musl-dev 
WORKDIR /usr/src/kon
COPY . .
RUN cargo fetch && cargo build -r

FROM alpine:3.19@sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672bd4f27761f8e1ad6b
RUN apk add --no-cache openssl-dev libgcc
WORKDIR /kon
COPY --from=compiler /usr/src/kon/target/release/kon .
COPY --from=compiler /usr/src/kon/Cargo.toml .
CMD [ "./kon" ]
