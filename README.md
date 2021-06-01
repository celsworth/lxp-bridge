# lxp-bridge

> **warning: this is under development and not ready for general consumption!**

> more docs and features coming soon.

lxp-bridge is a tool to get communications with a LuxPower inverter (commonly used with home-battery and solar setups) onto your MQTT network.

It allows you to control your inverter locally without any dependence on the manufacturer's own servers in China.

This builds on my earlier project [Octolux](https://github.com/celsworth/octolux), but where that attempted to be an all-in-one solution, this is a bit more tightly defined and doesn't attempt any control or intelligence of its own. This is simply a bridge from the inverter to MQTT. You get to do all the control on your own, from node-red or your own scripts or whatever.


## Installation

A range of binaries are provided on the Releases page, otherwise you can compile it yourself. It's written in Rust.

  1. [Install Rust](https://www.rust-lang.org/tools/install)
  1. `git clone https://github.com/celsworth/lxp-bridge.git`
  1. `cd lxp-bridge`
  1. `cargo build`
  1. Look in `target/` for the binary, or `cargo run` it.


## Configuration

All configuration is done in a YAML config file; see example config.yaml.

## Usage

As the inverter sends out packets, the bridge will translate the interesting ones (ie not heartbeats) into MQTT messages, as follows.

First thing to note is there are two types of registers in the inverter:

  * holdings - read/write, storing settings
  * inputs - read-only, storing transient power data, temperatures, counters etc

### `lxp/hold/1`

1 is actually any number from 1 to 179.

These are unprocessed raw values, sent when the inverter tells us the contents of a register.  This is normally done in response to the inverter being asked for it (which you can do yourself with `lxp/cmd/read_hold/1`, see below).

In some cases, they require further processing to make much sense. For example, registers 2-6 contain the serial number, but it's returned as 5xu16 and needs separating into 10xu8 to match the result you'll see on the inverter's screen. Example 2; register 100 is the lead-acid discharge cut-out voltage, but is in 0.1V units, so divide by 10 to get Volts.


You will see a whole bunch of these if you press "Read" under the Maintain tab in the LXP Web Monitor; this is the website reading all the values from your inverter so it can fill in the form with current values.

### `lxp/inputs/1` (and 2, and 3)

These are JSON hashes of post-processed data. There are 3 of them just because that's how the inverter sends the data. They are sent at 3 minute intervals.

Not sure what determines the interval, and I'm pretty sure it used to be 2 minutes so this interval might be stored in a register somewhere?

TODO: think you can request these to be sent immediately, once I make `lxp/cmd/read_inputs` work..

TODO: document the JSON hashes.


### Commands

When you want lxp-bridge to do something, you send a message under `lxp/cmd/...`; responses to commands will be sent under `lxp/result/...` where ... is the same as the command you sent. So sending `lxp/cmd/set/ac_charge` will return a response under `lxp/result/ac_charge`. This will be `OK` or `FAIL` depending on the result.

*boolean* values recognised as `true` in payloads are `1`, `t`, `true`, `on`, `y`, and `yes`. They're all equivalent. Anything else will be interpreted as `false`.

*percent* values should be an integer between 0 and 100.


The following MQTT topics are recognised:

#### topic = `lxp/cmd/set/ac_charge`, payload = boolean

Send a boolean to this to enable or disable immediate AC Charging.


#### topic = `lxp/cmd/read/hold/1`, payload = empty

This is a pretty low-level command which you may not normally need.

Publishing an empty message to this will read the value of register 1.

The unprocessed reply will appear in `lxp/hold/1`. Depending on which register you're reading, this may need further post-processing to make sense.


