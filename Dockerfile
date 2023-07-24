FROM rust:slim-buster

RUN apt-get update && \
    apt-get install -y \
    apt-get -y install libpq-dev && \
    apt-get -y install 