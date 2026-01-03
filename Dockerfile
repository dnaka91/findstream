# syntax=docker/dockerfile:1.20
FROM --platform=$BUILDPLATFORM tonistiigi/xx:1.8.0 AS xx
FROM --platform=$BUILDPLATFORM rust:1.92 AS builder

COPY --from=xx / /

WORKDIR /volume
ENV RUSTFLAGS="-C target-feature=+crt-static"
ENV XX_VERIFY_STATIC=1

RUN apt-get update && \
    apt-get install -y clang lld

ARG TARGETPLATFORM

RUN xx-apt-get install -y xx-c-essentials

COPY --parents assets/ src/ templates/ Cargo.lock Cargo.toml ./

RUN xx-cargo build --release && \
    xx-verify target/$(xx-cargo --print-target-triple)/release/findstream && \
    mkdir dist && \
    cp target/$(xx-cargo --print-target-triple)/release/findstream dist/

FROM --platform=$BUILDPLATFORM alpine:3 AS newuser

RUN echo "findstream:x:1000:" > /tmp/group && \
    echo "findstream:x:1000:1000::/dev/null:/sbin/nologin" > /tmp/passwd

FROM scratch

COPY --from=builder /volume/dist/findstream /bin/
COPY --from=newuser /tmp/group /tmp/passwd /etc/

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

EXPOSE 8080
USER findstream

ENTRYPOINT ["/bin/findstream"]
