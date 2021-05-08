# lxp-bridge

lxp-bridge is a tool to get communications with a LuxPower inverter (commonly used with home-battery and solar setups) onto your MQTT network.

It allows you to control your inverter locally without any dependence on the manufacturer's own servers in China.

This builds on my earlier project [Octolux](https://github.com/celsworth/octolux), but where that attempted to be an all-in-one solution, this is a bit more tightly defined and doesn't attempt any control or intelligence of its own. This is simply a bridge from the inverter to MQTT. You get to do all the control on your own, from node-red or your own scripts or whatever.


## Installation

A range of binaries are provided on the Releases page, otherwise you can compile it yourself. It's written in Rust.

  1. [Install Rust](https://www.rust-lang.org/tools/install)
  1. `git clone https://github.com/celsworth/octolux.git`
  1. `cargo build`
  1. Look in `target/` for the binary, or `cargo run` it.


## Configuration

All configuration is done in a YAML config file; see example config.yaml.
