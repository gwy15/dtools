# build
FROM rust:slim-buster as builder
WORKDIR /code
COPY . .
RUN echo ${PWD} && cargo b --release && ls target/release

# 
FROM debian:buster-slim
WORKDIR /code
COPY --from=builder target/release/signer /bin/signer
ENTRYPOINT [ "/bin/signer" ]
CMD ["--task=all"]
