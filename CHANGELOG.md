# 0.13.0 - 27th October 2023

* **BREAKING CHANGE**: Simplify scheduler.timesync configuration to appease HA (#209)
* Attempt to fix unsigned maths overflow (#211)
* Expose max_chg_curr and max_dischg_curr to HA (#212)


# 0.12.0 - 29th September 2023

* Add more sensors to HomeAssistant autodiscovery (#181, #194, @Sboshoff76)
* Add `p_battery` and `p_grid` inputs keys to show net power flows (#183)
* Avoid floating point maths oddities in e_pv_day and e_pv_all calculations (#185)
* Add internal_fault/warning_code/fault_code keys (#189, #190, #191)
* Revert to unsigned integers for inverter registers/values (#196)
* Fix charge_priority_en value in hold/21/bits MQTT message (#201)


# 0.11.0 - 16th July 2023

* Fix crash due to signed integer overflow when saving inputs to InfluxDB (#161, @dgcartersa)
* Add HomeAssistant add-on (#167, @apbarratt)
* Add loglevel option to config.yaml (#168)
* Add ac_first time register functionality (#171)


# 0.10.0 - 20th April 2023

* Fix crash in scheduler during DST transition times (#107)
* Add read individual input command and optional publishing of individual input registers (#111)
* [Internal Cleanup] Use signed integers for inverter registers/values (#115)
* Add WriteParam functionality (`lxp/cmd/all/set/param/X`) (#117)
* Decode bits in holding registers 21 and 110 and publish to `lxp/$datalog/hold/21/bits` (#119)
* Better HomeAssistant discovery message structure (#120, @unreadablename)
* Add MQTT messages to easily read/set time registers (#123)
* Add missing `lxp/cmd/$datalog/set/forced_discharge` (#125)
* Add more HA discovery sensors (#128, @unreadablename)
* Add MQTT LWT and use it in HA discovery messages (#129, #130)
* Add AC Charge/Charge Priority/Forced Discharge switches to HA discovery (#127)
* Remove v_pv inputs key (#135)
* Remove mqtt.homeassistant.sensors configuration option (#132, @lupine)
* Add HA discovery messages for number controls (AC Charge Cutoff % etc) (#132, @lupine)
* Fix crash in timesync during DST transition times (#153)
* Add option to send holding registers on startup (#147, @lupine)
* Add HomeAssistant time control discovery messages (#143, @lupine)
* Retain holding and parameter register messages (#154, @lupine)


# 0.9.0 - 2nd November 2022

* Fix incorrect FAIL MQTT reply on ReadParam commands (#93)
* Second attempt at ignoring unknown ReadInputs registers (#95)
* Update HA discovery to use newer "all" MQTT message (#98, @excieve)
* Exit on receipt of SIGTERM or SIGINT (#99, @kaitlinsm)
* Add support for replying to inverter heartbeats (#106)


# 0.8.0 - 1st September 2022

* Publish MQTT discovery packets with Retain bit set (#86)
* Be more tolerant of unknown ReadInputs registers (#89)
* Add missing p_eps and s_eps fields to ReadInput1 (#91)
* Ignore unhandled WriteParam (tcp_function=196) packets (#92)


# 0.7.0 - 26th June 2022

* Add Postgres/MySQL/SQLite support (#44, #45, #47)
* Use more meaningful labels for HomeAssistant autodiscovery (#55, @chriscn)
* Allow enabling individual HomeAssistant discovery sensors (#56)
* Support combined inputs data packet found in newer firmwares (#65, #82)
* Add internal WriteMulti packet support (not exposed to MQTT yet) (#68)
* Add scheduled tasks framework; first one is synchronize inverter clock (disabled by default) (#70)
* Log warning message when configured serial numbers don't match packets we receive from inverter (#78)
* Fix rare startup crash if inverter is in the middle of sending inputs (#80)


# 0.6.0 - 26th February 2022

* Merge input data packets into one hash when publishing to MQTT and InfluxDB (#36)
* Fix crash when InfluxDB was disabled (#42)
* Fix InfluxDB being inadvertently disabled when only MQTT should have been (#42)


# 0.5.1 - 2nd November 2021

* No functional changes; fix Windows build by bumping rumqttc crate to 0.10.0


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
