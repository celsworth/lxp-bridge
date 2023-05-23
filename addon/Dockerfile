ARG BUILD_VERSION
ARG BUILD_ARCH

FROM celsworth/lxp-bridge:${BUILD_VERSION} AS build
FROM ghcr.io/hassio-addons/base/${BUILD_ARCH}:stable

RUN "CP /data/options.json /etc/config.yaml" 