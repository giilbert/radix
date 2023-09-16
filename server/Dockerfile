FROM rust:1.70-alpine3.18 as builder

WORKDIR /usr/src
RUN apk add --no-cache musl-dev libressl-dev

RUN USER=root cargo new radix_server

WORKDIR /usr/src/radix_server

COPY Cargo.toml ./

RUN cargo build --release \
    && rm -rf src

ADD . .
# update mtimes https://github.com/rust-lang/cargo/issues/7181#issuecomment-515260460
RUN touch src/main.rs

RUN cargo build --release \
    && mv target/release/radix_server /bin \
    && rm -rf /usr/src/radix_server

FROM alpine:3.18

RUN apk add --no-cache libressl-dev ca-certificates-bundle bash
COPY --from=builder /bin/radix_server /bin/radix_server
ENV RUST_LOG=info

CMD ["/bin/radix_server"]
