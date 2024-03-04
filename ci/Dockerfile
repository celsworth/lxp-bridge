FROM --platform=$BUILDPLATFORM rust:1.71.1 as builder
ARG TARGETPLATFORM
WORKDIR /usr/src/lxp-bridge
COPY Cargo.toml .
COPY Cargo.lock .
COPY .cargo .cargo
COPY src src
COPY db db
COPY ci ci
RUN ./ci/docker-buildx-platform-setup.sh $TARGETPLATFORM
ENV OPENSSL_LIB_DIR /openssl-1.1.1s
ENV OPENSSL_INCLUDE_DIR /openssl-1.1.1s/include
RUN rustup target add $(cat /rust_target.txt)
RUN cargo install --path . --target $(cat /rust_target.txt)


FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libssl1.1 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/lxp-bridge /usr/local/bin/lxp-bridge
ENTRYPOINT ["lxp-bridge", "-c", "/etc/config.yaml"]
