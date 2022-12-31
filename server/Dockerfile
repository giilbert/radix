FROM rust:1.66 as builder

WORKDIR /usr/src
RUN USER=root cargo new radix_server

WORKDIR /usr/src/radix_server

COPY Cargo.toml Cargo.lock ./

RUN cargo build --release \
    && rm -rf src

ADD . .
# update mtimes https://github.com/rust-lang/cargo/issues/7181#issuecomment-515260460
RUN touch src/main.rs

RUN cargo build --release \
    && mv target/release/radix_server /bin \
    && rm -rf /usr/src/radix_server

FROM debian:latest

RUN apt-get update \
    && apt-get install -y ca-certificates \
    && rm -rf rm -rf /var/lib/apt/lists/*
COPY --from=builder /bin/radix_server /bin/radix_server

CMD ["/bin/radix_server"]
