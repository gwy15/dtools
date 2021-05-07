# build
FROM rust:slim-buster as builder
WORKDIR /code
COPY . .
RUN cargo b --release

# 
FROM debian:buster-slim
WORKDIR /code
COPY --from=builder /code/target/release/signer /bin/signer
ENTRYPOINT [ "/bin/signer" ]
CMD ["--task=all"]
