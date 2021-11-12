FROM rust:1.56-bullseye as builder
WORKDIR /usr/src/grocery-list-api
COPY . .
RUN cargo install --path .


FROM debian:bullseye
COPY --from=builder /usr/local/cargo/bin/grocery-list-api /usr/local/bin/grocery-list-api
ENV ROCKET_ADDRESS=0.0.0.0
CMD ["grocery-list-api"]