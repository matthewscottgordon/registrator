FROM rust:1.56-bullseye as builder
WORKDIR /usr/src/registrator
COPY Cargo.toml .
COPY Rocket.toml .
COPY src ./src
RUN cargo install --path .


FROM debian:bullseye
COPY --from=builder /usr/local/cargo/bin/registrator /usr/local/bin/registrator
COPY Rocket.toml /
CMD ["registrator"]