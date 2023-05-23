ARG BUILD_VERSION

FROM celsworth/lxp-bridge:${BUILD_VERSION} AS build

RUN cargo install yj
RUN yj /data/options.json -o /etc/config.yaml