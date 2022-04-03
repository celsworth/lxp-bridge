# FAQ

## I'm seeing power/energy flow readings coming out of the inverter, but controlling it does not work?

This is probably an incorrect inverter serial number / datalog serial number in the configuration. These are required to *send* data to the inverter, but are irrelevant for just listening to the inverter.

Easiest place to find your values is check the LuxPower Web Portal and check your inverter under Configuration -> Inverters.


## I'm seeing "response '204 No Content'" responses in the logs from InfluxDB

This is normal. 204 is the expected response from InfluxDB when it has stored a datapoint.

## Which binary do I use?

* `aarch64-apple-darwin` - macOS M1 (arm64)
* `aarch64-unknown-linux-gnu` - 64bit ARM Linux, eg Raspberry Pi 3 or 4 with 64bit OS.
* `arm-unknown-linux-gnueabihf` - 32bit ARM Linux, eg Raspberry Pi 1 or 2.
* `x86_64-apple-darwin` - macOS x86
* `x86_64-pc-windows-msvc` - Windows
* `x86_64-unknown-linux-gnu` - 64bit x86 Linux
