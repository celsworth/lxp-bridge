ARG BUILD_VERSION
ARG BUILD_FROM

FROM celsworth/lxp-bridge:${BUILD_VERSION} AS lxp-bridge

FROM $BUILD_FROM
COPY --from=lxp-bridge / /

RUN echo "$MIRROR/alpine/edge/community" >> /etc/apk/repositories
RUN apk update
RUN apk add yq

COPY run.sh /
RUN chmod a+x /run.sh

CMD [ "/run.sh" ]