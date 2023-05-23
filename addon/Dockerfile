ARG BUILD_VERSION
ARG BUILD_FROM

FROM $BUILD_FROM as base

FROM mikefarah/yq as json-to-yaml
COPY --from=base /data/options.json /data/options.json
RUN yq -P /data/options.json > /data/config.yaml

FROM celsworth/lxp-bridge:${BUILD_VERSION} AS lxp-bridge
COPY --from=json-to-yaml /data/config.yaml /etc/config.yaml

FROM base
COPY --from=lxp-bridge / /