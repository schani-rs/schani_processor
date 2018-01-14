FROM rust:1.23.0
WORKDIR /usr/src/myapp
COPY . .
RUN cargo build --release

FROM jamesnetherton/rawtherapee:latest
USER rawtherapee
COPY --from=0 /usr/src/myapp/target/release/processor /usr/local/bin

ENTRYPOINT ["processor"]
