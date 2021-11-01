# 0.5.0 - 1st November 2021

* Fix "Channel closed" crash when MQTT is disabled (#31)
* Fix: Send missing MQTT lxp/hold/XX message with new register value on receipt of a WriteSingle packet (#32)


# 0.4.0 - 12th October 2021

* Fix enabling/disabling AC Charge ignoring previous register value (#27)


# 0.3.0 - 6th September 2021

* Add support for Home Assistant MQTT discovery - power flow sensors only for now (#26)


# 0.2.0 - 3rd July 2021

* Add `lxp/cmd/{datalog}/read/inputs/{n}` functionality - read input registers on demand (#16)
* Add TCP keepalives to inverter connections (#18)
* Change `time` field of input packets from ISO8601 string to integer unix timestamp for better node-red compatibility (#17)
* Fix potential hang in inverter packet processing (#16)


# 0.1.0 - 24th June 2021

* Initial release
