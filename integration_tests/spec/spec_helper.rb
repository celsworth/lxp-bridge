# frozen_string_literal: true

require 'mqtt'
require 'rspec'
require 'json'
require 'sequel'

require_relative 'support/inverter'

RSpec.configure do |config|
  config.filter_run focus: true
  config.run_all_when_everything_filtered = true

  config.add_setting :mqtt
  config.before(:suite) do
    RSpec.configuration.mqtt = MQTT::Client.connect('mqtt://localhost')
  end

  config.around do |example|
    @inverter = Inverter.new

    @messages = []

    RSpec.configuration.mqtt.subscribe('#')
    sub_thread = Thread.new do
      RSpec.configuration.mqtt.get do |topic, data|
        data = begin
          JSON.parse(data)
        rescue JSON::ParserError
          data
        end

        @messages << { topic: topic, data: data }
      end
    end

    example.run

    sub_thread.kill
    RSpec.configuration.mqtt.unsubscribe('#')
  end

  def fixture(file)
    JSON.parse(File.read("fixtures/#{file}.json"))
  end

  # run the given block until it returns a truthy value, with a sleep between each attempt
  def with_retries(retries: 50)
    r = nil

    retries.times do
      r = yield
      break if r

      sleep 0.01
    end

    r
  end

  def publish(topic, payload)
    RSpec.configuration.mqtt.publish(topic, payload)
  end

  def wait_for_mqtt(topic)
    message = with_retries { @messages.find { |h| h[:topic] == topic } }
    message&.fetch(:data)
  end

  def inverter # rubocop:disable Style/TrivialAccessors
    @inverter
  end

  def sqlite_inputs_table
    @sqlite_inputs_table ||= Sequel.sqlite('tmp/db/lxp.db')[:inputs]
  end
end
