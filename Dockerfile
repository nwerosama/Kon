FROM scratch as base
WORKDIR /builder
COPY . .

FROM alpine:edge
LABEL org.opencontainers.image.source="https://git.toast-server.net/toast/Kon"
RUN apk add --no-cache libgcc fluidsynth
WORKDIR /kon
COPY --from=builder /builder/target/x86_64-unknown-linux-musl/release/kon .
CMD [ "./kon" ]
