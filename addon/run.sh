#!/usr/bin/with-contenv bashio

bashio::log.info "Creating lxp-bridge config from options..."

yq -P /data/options.json > /etc/config.yaml

bashio::log "Done."

bashio::log.info "Starting lxp-bridge..."

/usr/local/bin/lxp-bridge -c /etc/config.yaml