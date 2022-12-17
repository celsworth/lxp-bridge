# lxp-bridge integration test suite

This is a new experimental idea to add end-to-end testing of lxp-bridge from an external standpoint.

A docker-compose file starts up required services (an inverter, MQTT, maybe soon a database etc) and an instance of lxp-bridge, then a Ruby rspec suite triggers various messages between MQTT and the inverter to check how lxp-bridge handles it.

* `docker-compose up --build`
* `bundle install`
* `bundle exec rspec`

Limitations:

* a persistent MQTT instance is used for the entire suite, so a test that creates a retained message on the broker will cause subsequent tests to see the same retained message. This includes the retained HA discovery messages on lxp-bridge startup.
