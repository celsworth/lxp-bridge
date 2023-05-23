ARG BUILD_VERSION

FROM celsworth/lxp-bridge:${BUILD_VERSION} AS build
FROM mikefarah/yq

RUN yq -P /data/options.json > /etc/config.yaml