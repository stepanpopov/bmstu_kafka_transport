ARG RUSTFLAGS='-C target-feature=+crt-static'
ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-gnu

FROM rust:1.76 as builder

# create a new workspace
RUN USER=root mkdir transport
WORKDIR /transport

# copy workspace manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# copy workspace members manifests
RUN USER=root cargo new --lib common
COPY ./common/Cargo.toml ./common/Cargo.toml

RUN USER=root cargo new --bin consume
COPY ./consume/Cargo.toml ./consume/Cargo.toml

RUN USER=root cargo new --bin split
COPY ./split/Cargo.toml ./split/Cargo.toml

RUN USER=root cargo new --bin produce
COPY ./produce/Cargo.toml ./produce/Cargo.toml

# for build rdkafka 
RUN USER=root apt update && apt install cmake -y

ARG RUSTFLAGS
ARG CARGO_BUILD_TARGET

ENV RUSTFLAGS=${RUSTFLAGS}
ENV CARGO_BUILD_TARGET=${CARGO_BUILD_TARGET}

# this build step will cache your dependencies

RUN cargo build --release -v

# copy your source tree
COPY ./common/src ./common/src
COPY ./consume/src ./consume/src
COPY ./split/src ./split/src
COPY ./produce/src ./produce/src

RUN rm ./target/${CARGO_BUILD_TARGET}/release/deps/libcommon*
RUN cargo build --release -v

RUN ldd ./target/${CARGO_BUILD_TARGET}/release/split && echo 'aaa' && echo ${RUSTFLAGS} && cat ./split/src/main.rs

# final
RUN echo FINAL


# consume
FROM scratch as consume
ARG CARGO_BUILD_TARGET
COPY --from=builder ./transport/target/${CARGO_BUILD_TARGET}/release/consume /usr/local/bin/consume

ENTRYPOINT [ "consume" ]
CMD ["--brokers=kafka1:9092", "--topic=test", "--group-id=akanev", "--receive_url=http://receive:8082/receive"]
LABEL service=consume


# scratch
FROM scratch as produce
ARG CARGO_BUILD_TARGET
COPY --from=builder ./transport/target/${CARGO_BUILD_TARGET}/release/produce /usr/local/bin/produce

ENV LISTEN=0.0.0.0:8002

ENTRYPOINT [ "produce" ]
CMD ["--brokers=kafka1:9092", "--topic=test"]
LABEL service=produce


# split
FROM scratch as split
ARG CARGO_BUILD_TARGET
COPY --from=builder /transport/target/${CARGO_BUILD_TARGET}/release/split /usr/local/bin/split

ENV LISTEN=0.0.0.0:8000

ENTRYPOINT [ "split" ]
CMD ["--code-service-url=http://code:8081/code", "--chunk-byte-size=200"]
LABEL service=split
