FROM rust:1.90.0-alpine

RUN apk add --no-cache \
    build-base \
    musl-dev \
    perl \
    make \
    iputils \
    procps-ng \
    vim \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    git \
    python3 \
    py3-pip \
    bash \
    cmake \
    linux-headers