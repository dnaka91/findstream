FROM rust:1.79 as builder

WORKDIR /volume

RUN apt-get update && \
    apt-get install -y --no-install-recommends musl-tools && \
    rustup target add x86_64-unknown-linux-musl && \
    cargo init --bin

COPY Cargo.lock Cargo.toml ./

RUN cargo build --release --target x86_64-unknown-linux-musl

COPY assets/ assets/
COPY src/ src/
COPY templates/ templates/

RUN touch src/main.rs && cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3 as newuser

RUN echo "findstream:x:1000:" > /tmp/group && \
    echo "findstream:x:1000:1000::/dev/null:/sbin/nologin" > /tmp/passwd

FROM scratch

COPY --from=builder /volume/target/x86_64-unknown-linux-musl/release/findstream /bin/
COPY --from=newuser /tmp/group /tmp/passwd /etc/

EXPOSE 8080
USER findstream

ENTRYPOINT ["/bin/findstream"]
