FROM scratch AS base
WORKDIR /builder
COPY . .

FROM adelielinux/adelie:1.0-beta6
LABEL org.opencontainers.image.source="https://git.toast-server.net/nwerosama/Kon"
RUN apk add --no-cache libgcc
WORKDIR /kon
COPY --from=base /builder/target/x86_64-unknown-linux-musl/release/kon .
CMD [ "./kon" ]
