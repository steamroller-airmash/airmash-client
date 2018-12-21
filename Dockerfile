
FROM clux/muslrust:nightly

WORKDIR /build
ADD Cargo.toml Cargo.lock /build/
RUN mkdir /build/src
RUN echo 'fn main() {}' > src/lib.rs

RUN cargo fetch
ADD src /build/src
RUN cargo build --release
RUN mkdir /artifacts
RUN mv target/x86_64-unknown-linux-musl/release/airmash-client /artifacts/airmash-client

FROM library/debian:latest

RUN apt-get update && apt-get install ca-certificates tor curl -y
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs
#ENV RUST_BACKTRACE=1
COPY --from=0 /artifacts/airmash-client /airmash-client
ADD script /script

ENTRYPOINT ./script
