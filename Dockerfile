# build
FROM rust:slim as builder
WORKDIR /code
COPY . .
RUN cargo b --release

# 
FROM slim
WORKDIR /code
COPY --from=builder /code/target/release/signer /bin/signer
ENTRYPOINT [ "/bin/signer" ]
CMD ["--task=all"]
