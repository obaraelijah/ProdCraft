FROM rust:1.59.0

WORKDIR /app

RUN apt update && apt install lld clang -y

COPY . .

ENV key=value

ENV SQLX_OFFLINE true

ENTRYPOINT ["./target/release/prod_craft"]
