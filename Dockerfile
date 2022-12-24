#FROM funtoo/stage3-intel64-skylake
FROM rust:latest

RUN mkdir /usr/target

WORKDIR /usr/workspace

#ENV RUSTFLAGS="--cfg tokio_unstable"
ENV CARGO_HOME=/usr/cargo_home
ENV CARGO_TARGET_DIR=/usr/target

CMD ["cargo build","cargo run"]
