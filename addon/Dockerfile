ARG BUILD_VERSION
ARG BUILD_ARCH

FROM celsworth/lxp-bridge:${BUILD_VERSION} AS build
FROM ghcr.io/hassio-addons/base/${BUILD_ARCH}:stable

RUN cargo install yj
RUN yj /data/options.json -o /etc/config.yaml