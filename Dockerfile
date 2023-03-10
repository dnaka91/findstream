FROM rust:1.68 as builder

WORKDIR /volume

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN apt-get update && \
    apt-get install -y --no-install-recommends musl-tools=1.2.2-1 && \
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
