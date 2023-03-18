# lxp-bridge integration test suite

This is a new experimental idea to add end-to-end testing of lxp-bridge from an external standpoint.

The test suite supervises an instance of mosquitto, a (faked) inverter, and
lxp-bridge, sending messages and receiving replies over MQTT to verify
behaviour.

To run the suite:

* Ensure `docker` is available (it's used to run mosquitto in a container)
* `bundle install`
* `bundle exec rspec`

## Limitations

The suite is only as good as the fake inverter.
