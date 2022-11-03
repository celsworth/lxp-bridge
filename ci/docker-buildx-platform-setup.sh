#! /bin/bash

install_openssl() {
  cd /
  wget https://www.openssl.org/source/openssl-1.1.1s.tar.gz
  tar xzf openssl-1.1.1s.tar.gz
  cd openssl-1.1.1s && ./config shared && make
  #export OPENSSL_LIB_DIR=~/openssl-1.1.1s
  #export OPENSSL_INCLUDE_DIR=~/openssl-1.1.1s/include
}

case "$1" in
  "linux/amd64")
    echo x86_64-unknown-linux-gnu > /rust_target.txt
    ;;
  "linux/arm64")
    apt-get update && apt-get install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libc6-dev-arm64-cross
    echo aarch64-unknown-linux-gnu > /rust_target.txt
    export MACHINE=aarch64 ARCH=aarch64 CC=aarch64-linux-gnu-gcc
    ;;
  "linux/arm/v6")
    apt-get update && apt-get install -y gcc-arm-linux-gnueabihf libc6-dev-armel-cross
    echo arm-unknown-linux-gnueabihf > /rust_target.txt
    export MACHINE=arm ARCH=arm CC=arm-linux-gnueabihf-gcc
    ;;
  "linux/arm/v7")
    apt-get update && apt-get install -y gcc-arm-linux-gnueabihf libc6-dev-armel-cross
    echo armv7-unknown-linux-gnueabihf > /rust_target.txt
    export MACHINE=arm ARCH=arm CC=arm-linux-gnueabihf-gcc
    ;;
  *)
    exit 1
    ;;
esac

install_openssl
