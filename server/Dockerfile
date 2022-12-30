FROM rust:1.66 as builder
WORKDIR /usr/src/radix_server
COPY . .
RUN cargo install --path .

FROM debian
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/radix_server /usr/local/bin/radix_server
CMD ["radix_server"]
