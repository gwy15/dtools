# build
FROM rust:slim-buster as builder
WORKDIR /code
COPY . .
RUN cargo b --release && strip target/release/dtools

# 
FROM debian:buster-slim
WORKDIR /code
COPY --from=builder /code/target/release/dtools .
COPY --from=builder /code/log4rs.yml .
ENTRYPOINT [ "./dtools" ]
CMD []
