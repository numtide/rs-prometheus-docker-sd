FROM ekidd/rust-musl-builder as builder

WORKDIR /home/rust/

COPY . .
RUN cargo test
RUN cargo build --release

ENTRYPOINT ["./target/x86_64-unknown-linux-musl/release/rs-promotheus-docker-sd"]

FROM scratch
WORKDIR /home/rust/
COPY --from=builder /home/rust/target/x86_64-unknown-linux-musl/release/rs-promotheus-docker-sd .
ENTRYPOINT ["./rs-promotheus-docker-sd"]
