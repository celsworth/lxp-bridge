# lxp-bridge integration test suite

This is a new experimental idea to add end-to-end testing of lxp-bridge from an external standpoint.

A docker-compose file starts up required services (an inverter, MQTT, maybe soon a database etc) and an instance of lxp-bridge, then a Ruby rspec suite triggers various messages between MQTT and the inverter to check how lxp-bridge handles it.

Currently you need to create an SQlite database placeholder in the correct place. lxp-bridge will initialise and run migrations on this empty file.

* `touch tmp/db/lxp.db`
* `docker-compose up --build`
* `bundle install`
* `bundle exec rspec`

Limitations:

* a persistent MQTT instance is used for the entire suite, so a test that creates a retained message on the broker will cause subsequent tests to see the same retained message. This includes the retained HA discovery messages on lxp-bridge startup.
* a failing test will probably leave lxp-bridge and supporting services in an indeterminate state (for example, expecting data from the inverter) so sometimes you need to restart docker-compose and start over. For CI this isn't an issue, and rspec's fail-fast negates a lot of the problem (subsequent tests would probably all fail too).
