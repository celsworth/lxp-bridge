# lxp-bridge

> **warning: this is under development and not ready for general consumption!**

> more docs and features coming soon.

lxp-bridge is a tool to communicate with a LuxPower inverter (commonly used with home-battery and solar setups).

It allows you to monitor and control your inverter locally without any dependence on the manufacturer's own servers in China.

This builds on my earlier project [Octolux](https://github.com/celsworth/octolux), but where that attempted to be an all-in-one solution, this is a bit more tightly defined and doesn't attempt any control or intelligence of its own. This is simply a bridge from the inverter to commonly-used technologies (see below). You get to do all the control on your own, from node-red or your own scripts or whatever.

Currently, lxp-bridge bridges to:

* mqtt (push data for monitoring, listen to control commands)
* InfluxDB (push power data up for graphing etc)

In future, it might possibly run a HTTP server with endpoints to fetch power data or control the inverter via REST.


## Installation

A range of binaries are provided on the Releases page, otherwise you can compile it yourself. It's written in Rust.

  1. [Install Rust](https://www.rust-lang.org/tools/install)
  1. `git clone https://github.com/celsworth/lxp-bridge.git`
  1. `cd lxp-bridge`
  1. `cargo build`
  1. Look in `target/` for the binary, or `cargo run` it.


## Configuration

All configuration is done in a YAML config file; see example config.yaml.

Multiple inverters are supported via an array under the `inverters` key. Each one can be separately disabled if you want to temporarily stop connecting to one. Similarly, MQTT and InfluxDB can have `enabled = false` set to disable either output method.

## Basics

First thing to note is there are three types of registers:

  * holdings - read/write, storing settings
  * inputs - read-only, storing transient power data, temperatures, counters etc
  * params - read/write, these are actually on the datalog (the WiFi bit that plugs in) and currently all I think it does is set the interval at which inputs are broadcast.

Second thing is whenever the inverter receives a packet, it broadcasts the reply out to *all* connected clients. So you may see unprompted messages for holding 12/13/14 for instance; this is LuxPower in China occasionally requesting the time from your inverter (presumably so they can correct it if needs be).

## InfluxDB

lxp-bridge can publish power data (the contents of the `input` registers) to InfluxDB as they are received.

The database can be set in the configuration; the measurement table used is `inputs`. There will be a single tag of the inverter's datalog, and then fields which correspond with the same as the JSON data sent via MQTT.

Note that because the inverter sends the power data split across 3 packets, there will be 3 submissions to InfluxDB, each with slightly differing times (by about a second). This means all the data combined isn't an atomic snapshot of an instant in time, but in practise this shouldn't really matter.


## MQTT

As we receive packets from the inverter, we translate the interesting ones (ie not heartbeats) into MQTT messages, as follows.

### `lxp/{datalog}/hold/1`

1 is actually any number from 1 to 179.

These are unprocessed raw values, sent when the inverter tells us the contents of a register.  This is normally done in response to the inverter being asked for it (which you can do yourself with `lxp/cmd/{datalog}/read_hold/1`, see below).

In some cases, they require further processing to make much sense. For example, registers 2-6 contain the serial number, but it's returned as 5xu16 and needs separating into 10xu8 to match the result you'll see on the inverter's screen. Example 2; register 100 is the lead-acid discharge cut-out voltage, but is in 0.1V units, so divide by 10 to get Volts.

You will see a whole bunch of these if you press "Read" under the Maintain tab in the LXP Web Monitor; this is the website reading all the values from your inverter so it can fill in the form with current values.

### `lxp/{datalog}/inputs/1` (and 2, and 3)

These are JSON hashes of transient data. There are 3 of them just because that's how the inverter sends the data. They are sent at 5 minute intervals.

Not sure what determines the interval, and I'm pretty sure it used to be 2 minutes so this interval might be stored in a register somewhere?

TODO: think you can request these to be sent immediately, once I make `lxp/cmd/{datalog}/read_inputs` work..

TODO: document the JSON hashes.

### `lxp/{datalog}/params/0`

These are parameters stored on the datalog (the WiFi dongle), not the main inverter itself. The only parameter I'm aware of is 0 which appears to be the number of seconds between `inputs` broadcasts.

This area is a bit unknown - TODO for myself: try changing params/0 and see if the broadcast interval changes accordingly.


### Commands

When you want lxp-bridge to do something, you send a message under `lxp/cmd/...`; responses to commands will be sent under `lxp/result/...` where ... is the same as the command you sent. So sending `lxp/cmd/{datalog}/set/ac_charge` will return a response under `lxp/result/{datalog}/ac_charge`. This will be `OK` or `FAIL` depending on the result.

*boolean* values recognised as `true` in payloads are `1`, `t`, `true`, `on`, `y`, and `yes`. They're all equivalent. Anything else will be interpreted as `false`.

*percent* values should be an integer between 0 and 100.


The following MQTT topics are recognised:

#### topic = `lxp/cmd/{datalog}/read/hold/1`, payload = empty

This is a pretty low-level command which you may not normally need.

Publishing an empty message to this will read the value of inverter register 1.

The unprocessed reply will appear in `lxp/hold/1`. Depending on which register you're reading, this may need further post-processing to make sense.


#### topic = `lxp/cmd/{datalog}/set/hold/1`, payload = int

This is a pretty low-level command which you may not normally need.

Publishing to this will set the given register to the payload, which should be a 16-bit integer.


#### topic = `lxp/cmd/{datalog}/read/param/0`, payload = empty

This is a pretty low-level command which you may not normally need.

Publishing an empty message to this will read the value of datalog parameter 0.

The unprocessed reply will appear in `lxp/param/0`. Depending on which parameter you're reading, this may need further post-processing to make sense.

TODO: separate doc with known parameters? For now only 0 is known to work, which is the interval between inputs being published, in seconds.


#### topic = `lxp/cmd/{datalog}/set/ac_charge`, payload = boolean

Send a boolean to this to enable or disable immediate AC Charging (from the grid).


#### topic = `lxp/cmd/{datalog}/set/forced_discharge`, payload = boolean

Send a boolean to this to enable or disable immediate forced discharging.


#### topic = `lxp/cmd/{datalog}/set/charge_rate_pct`, payload = percent

Send an integer in the range 0-100 (%) to this to set the global system charge rate. 100% is full power (3.6kW or so generally).


#### topic = `lxp/cmd/{datalog}/set/discharge_rate_pct`, payload = percent

Send an integer in the range 0-100 (%) to this to set the global system discharge rate. 100% is full power (3.6kW or so generally).


#### topic = `lxp/cmd/{datalog}/set/ac_charge_rate_pct`, payload = percent

Send an integer in the range 0-100 (%) to this to set the charge rate when AC charging (from the grid). 100% is full power (3.6kW or so generally).


#### topic = `lxp/cmd/{datalog}/set/ac_charge_soc_limit_pct`, payload = percent

Send an integer in the range 0-100 (%) to this to set the battery SOC at which AC charging will stop.


#### topic = `lxp/cmd/{datalog}/set/discharge_cutoff_soc_limit_pct`, payload = percent

Send an integer in the range 0-100 (%) to this to set the battery SOC at which discharging will stop.


