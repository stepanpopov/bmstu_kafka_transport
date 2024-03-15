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

# this build step will cache your dependencies
RUN cargo build --release

# copy your source tree
COPY ./common/src ./common/src
COPY ./consume/src ./consume/src
COPY ./split/src ./split/src
COPY ./produce/src ./produce/src

# build for release
# RUN rm ./target/release/deps/common*
#RUN rm ./target/release/deps/split*
#RUN rm ./target/release/deps/consume*
#RUN rm ./target/release/deps/produce*

RUN cargo build --release

# final
RUN echo FINAL

ENV BROKERS=localhost:9094
ENV TOPIC=test
ENV COUNSUMER_GROUP_ID=aaa

## consume
ENV CONSUME_RECEIVE_SERVICE_URL=http://localhost:8082/receive
ENV COUNSUMER_GROUP_ID=aaa

FROM scratch as consume
COPY --from=builder ./transport/target/release/consume ./
CMD ./consume --brokers=${BROKERS} --topic=${TOPIC} --group-id=${COUNSUMER_GROUP_ID} --receive_url=${CONSUME_RECEIVE_SERVICE_URL}
LABEL service=consume

## produce
ENV LISTEN_PRODUCE=0.0.0.0:8002

FROM scratch as produce
COPY --from=builder ./transport/target/release/produce ./
CMD LISTEN=${LISTEN_PRODUCE} ./produce --brokers=${BROKERS} --topic=${TOPIC}
LABEL service=produce


## split
ENV CHUNK_BYTE_SIZE=200
ENV SPLIT_CODE_SERVICE_URL=http://localhost:8081/code
ENV LISTEN_SPLIT=0.0.0.0:8000

FROM scratch as split
COPY --from=builder ./transport/target/release/split ./
CMD LISTEN=${LISTEN_SPLIT} ./split --code-service-url=${SPLIT_CODE_SERVICE_URL} --chunk_byte_size=${CHUNK_BYTE_SIZE}
LABEL service=split
