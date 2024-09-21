FROM scratch AS base
WORKDIR /builder
COPY . .

FROM alpine:3.20
LABEL org.opencontainers.image.source="https://git.toast-server.net/toast/Kon"
RUN apk add --no-cache libgcc fluidsynth
WORKDIR /kon
COPY --from=base /builder/target/x86_64-unknown-linux-musl/release/kon .
CMD [ "./kon" ]
