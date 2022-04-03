# lxp-bridge

> **warning: this is under development and not ready for general consumption!**

> more docs and features coming soon.

lxp-bridge is a tool to communicate with a LuxPower inverter (commonly used with home-battery and solar setups), written in Rust.

It allows you to monitor and control your inverter locally without any dependence on the manufacturer's own servers in China.

This builds on my earlier project [Octolux](https://github.com/celsworth/octolux), but where that attempted to be an all-in-one solution, this is a bit more tightly defined and doesn't attempt any control or intelligence of its own. This is simply a bridge from the inverter to commonly-used technologies (see below). You get to do all the control on your own, from node-red or your own scripts or whatever.

Currently, lxp-bridge bridges to:

* mqtt (publish data for monitoring, subscribe to control commands)
* InfluxDB (push power data for graphing etc)
* Postgres, MySQL, and SQLite (save power data for long term storage)

There's also support for publishing Home Assistant MQTT discovery packets, which is enabled by default. Home Assistant should automatically detect your inverter as a bunch of sensors (power/energy flows only for now) the first time you start lxp-bridge.

In future, it might possibly run a HTTP server with endpoints to fetch power data or control the inverter via REST.


## Installation

A range of binaries are provided on the [Releases](https://github.com/celsworth/lxp-bridge/releases) page (if you're unsure which you need, check the [faq](doc/faq.md)), otherwise you can compile it yourself.

Installation is not trivial but hopefully these steps will get you going:

  1. [Install Rust](https://www.rust-lang.org/tools/install)
  1. `git clone https://github.com/celsworth/lxp-bridge.git`
  1. `cd lxp-bridge`
  1. `make`
  1. `sudo make install` will install the binary to `/usr/local/bin/lxp-bridge`
  1. `sudo mkdir /etc/lxp-bridge`
  1. `sudo cp config.yaml.example /etc/lxp-bridge/config.yaml`
  1. Edit /etc/lxp-bridge/config.yaml to suit your configuration
  1. Use [systemd/lxp-bridge.service](systemd/lxp-bridge.service) to get it running:
     1. `sudo cp systemd/lxp-bridge.service /etc/systemd/system`
     1. `sudo systemctl daemon-reload`
     1. `sudo systemctl start lxp-bridge`

Alternatively you can try running it in Docker:

  1. You'll need Docker
  1. Create a `config.yaml` by copying the example from this repo and editing.
  1. `docker run --init --rm --mount type=bind,source=${PWD}/config.yaml,target=/etc/config.yaml celsworth/lxp-bridge`

The final step should leave you with a running lxp-bridge that maps the config in the current directory into the Docker image.


## Configuration

All configuration is done in a YAML config file; see [config.yaml.example](config.yaml.example).

Multiple inverters are supported via an array under the `inverters` key. Each one can be separately disabled if you want to temporarily stop connecting to one. Similarly, MQTT and InfluxDB can have `enabled = false` set to disable either output method.

See [inverter_setup.md](doc/inverter_setup.md) for first-time setup of the inverter; it needs to listen on a port so lxp-bridge can connect to it.

## Basics

First thing to note is there are three types of registers:

  * holdings - read/write, storing settings. MQTT topics use `hold` for these.
  * inputs - read-only, storing transient power data, temperatures, counters etc.
  * params - read/write, these are actually on the datalog (the WiFi bit that plugs in) and currently all I think it does is set the interval at which inputs are broadcast.

Second thing is whenever the inverter receives a packet, it broadcasts the reply out to *all* connected clients. So you may see unprompted messages for holding 12/13/14 for instance; this is LuxPower in China occasionally requesting the time from your inverter (presumably so they can correct it if needs be).

## InfluxDB

lxp-bridge can publish power data (the contents of the `input` registers) to InfluxDB as they are received. Currently only InfluxDB v1 is supported.

The database can be set in the configuration; the measurement table used is `inputs`. Note you need to create the database yourself, lxp-bridge does not currently do it. The table will be automatically created though.

There will be a single tag of the inverter's datalog, and then fields which correspond with the same as the JSON data sent via MQTT - see [inputs.md](doc/inputs.md) for details.

## Databases

See [database.md](doc/database.md).

## MQTT

As we receive packets from the inverter, we translate the interesting ones (ie not heartbeats) into MQTT messages, as follows.

Note that everything documented here is under `lxp`, but this namespace can be changed in the config file.

`{datalog}` will be replaced with the datalog of the inverter that sent the message; this is because multiple inverters are supported, so you need to know which one a message came from.

### `lxp/{datalog}/hold/1`

1 is actually any number from 1 to 179.

These are unprocessed raw values, sent when the inverter tells us the contents of a register.  This is normally done in response to the inverter being asked for it (which you can do yourself with `lxp/cmd/{datalog}/read_hold/1`, see below).

In some cases, they require further processing to make much sense. For example, registers 2-6 contain the serial number, but it's returned as 5xu16 and needs separating into 10xu8 to match the result you'll see on the inverter's screen. Example 2; register 100 is the lead-acid discharge cut-out voltage, but is in 0.1V units, so divide by 10 to get Volts.

You will see a whole bunch of these if you press "Read" under the Maintain tab in the LXP Web Monitor; this is the website reading all the values from your inverter so it can fill in the form with current values.

### `lxp/{datalog}/inputs/all` (and 1, and 2, and 3)

These are hashes of transient data. The inverter sends these across 3 packets, which are directly mapped into JSON and published in `lxp/{datalog}/inputs/1`, `../2` and `../3`. From lxp-bridge v0.6.0, there is also an `../all` message which combines all three into a single hash of data.

Eventually (not before lxp-bridge v1.0) the individual messages may be removed in favour of the new `all` message. Please prefer use of the `all` message in favour of the 1/2/3 messages in new projects.

These are sent at 5 minute intervals. You can also read them on demand, see `lxp/cmd/{datalog}/read/inputs/1` below.

Also see [inputs.md](doc/inputs.md) for details of the JSON data hashes.


### `lxp/{datalog}/params/0`

These are parameters stored on the datalog (the WiFi dongle), not the main inverter itself. The only parameter I'm aware of is 0 which appears to be the number of seconds between `inputs` broadcasts.

This area is a bit unknown - TODO for myself: try changing params/0 and see if the broadcast interval changes accordingly.


### Commands

When you want lxp-bridge to do something, you send a message under `lxp/cmd/...`. There's two types of response depending what you're doing.

There's a result topic which is `OK` or `FAIL`; for example sending `lxp/cmd/{datalog}/set/ac_charge` will return `lxp/result/{datalog}/ac_charge`.

If you're reading a register then you get the result and additionally the value(s) will be sent in the same topic as above, so for example sending `lxp/cmd/{datalog}/read/inputs/1` will get you a `lxp/result/{datalog}/read/inputs/1` *and* a `lxp/{datalog}/inputs/1` with all the usual data.

*boolean* values recognised as `true` in payloads are `1`, `t`, `true`, `on`, `y`, and `yes`. They're all equivalent. Anything else will be interpreted as `false`.

*percent* values should be an integer between 0 and 100.


The following MQTT topics are recognised:

#### topic = `lxp/cmd/{datalog}/read/inputs/1`, payload = empty

This prompts the inverter to immediately publish a set of input registers. These get published every few minutes anyway but with this you can read them on-demand.

`1` can be 1 - 3. It is not yet possible to read all the registers at once, as seen in `lxp/{datalog}/inputs/all`.

See [inputs.md](doc/inputs.md) for details of the JSON data hashes.


#### topic = `lxp/cmd/{datalog}/read/hold/1`, payload = optional int

This is a pretty low-level command which you may not normally need.

Publishing to this will read the value of inverter holding register 1. The payload is optionally the number of registers to read, with a default of 1 if empty.

The unprocessed reply will appear in `lxp/{datalog}/hold/1`. Depending on which register you're reading, this may need further post-processing to make sense.

If you read multiple registers they will appear in their own replies, `lxp/{datalog}/hold/12`, `lxp/{datalog}/hold/13` etc.


#### topic = `lxp/cmd/{datalog}/set/hold/1`, payload = int

This is a pretty low-level command which you may not normally need.

Publishing to this will set the given register to the payload, which should be a 16-bit integer.

The unprocessed reply will appear in `lxp/{datalog}/hold/1`. Depending on which register you're reading, this may need further post-processing to make sense.

TODO: doc known registers

#### topic = `lxp/cmd/{datalog}/read/param/0`, payload = empty

This is a pretty low-level command which you may not normally need.

Publishing an empty message to this will read the value of datalog parameter 0.

The unprocessed reply will appear in `lxp/{datalog}/param/0`. Depending on which parameter you're reading, this may need further post-processing to make sense.

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


