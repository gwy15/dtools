# build
FROM rust:slim-buster as builder
WORKDIR /code
COPY . .
RUN echo ${PWD} && cargo b --release && ls target/release

# 
FROM debian:buster-slim
WORKDIR /code
COPY --from=builder /code/target/release/dtools .
COPY --from=builder /code/log4rs.yml .
ENTRYPOINT [ "./dtools" ]
CMD []
