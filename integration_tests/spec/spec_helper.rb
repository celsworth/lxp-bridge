# frozen_string_literal: true

require 'mqtt'
require 'rspec'
require 'json'
require 'sequel'

require_relative 'support/inverter'

RSpec.configure do |config|
  config.filter_run focus: true
  config.run_all_when_everything_filtered = true

  # Set up an MQTT server
  config.around(:each) do |example|
    name = "mosquitto-lxp-bridge-tests-#{Process.pid}"
    Kernel.system(
      *%W[docker run --detach --rm -p 1883:1883 -p 9001:9001 --name #{name} eclipse-mosquitto:1.6],
      out: '/dev/null'
    )

    begin
      wait_for_socket('localhost', 1883)
      example.run
    ensure
      Kernel.system(*%W[docker stop #{name}], out: '/dev/null')
    end
  end

  # Set up an MQTT client, for expectations
  config.around(:each) do |example|
    @mqtt_client = MQTT::Client.connect('mqtt://localhost:1883')
    @messages = []

    mqtt_client.subscribe('#')

    thread = Thread.new do
      mqtt_client.get do |topic, raw_data|
        data = begin
          JSON.parse(raw_data)
        rescue JSON::ParserError
          raw_data
        end

        @messages << { topic: topic, data: data }
      end
    end

    begin
      example.run
    ensure
      mqtt_client.disconnect
      thread.kill
      thread.join
    end
  end

  config.around(:each) do |example|
    @inverter = Inverter.new
    inverter.run

    begin
      example.run
    ensure
      inverter.close
    end
  end

  # Set up an lxp-bridge instance
  config.before(:suite) do
    Kernel.system(*%w[cargo build --manifest-path ../Cargo.toml])
  end

  config.around(:each) do |example|
    FileUtils.rm_rf('tmp/db/lxp.db')
    FileUtils.touch('tmp/db/lxp.db')

    pid = Process.spawn(
      { 'RUST_LOG' => 'FATAL' },
      *%w[../target/debug/lxp-bridge -c lxp-bridge.config.yaml]
    )

    # Wait for lxp-bridge to appear
    # wait_for_socket('localhost', 8086)
    sleep(1)

    begin
      example.run
    ensure
      Process.kill('TERM', pid)
      Process.wait(pid)
    end
  end

  def fixture(file)
    JSON.parse(File.read("fixtures/#{file}.json"))
  end

  # run the given block until it returns a truthy value, with a sleep between each attempt
  def with_retries(retries: 1000)
    r = nil

    retries.times do
      r = yield
      break if r

      sleep 0.01
    end

    r || raise('Retries exceeded')
  end

  def wait_for_socket(host, port)
    with_retries do
      TCPSocket.new(host, port).close
      :ok
    end
  end

  # rubocop:disable Style/TrivialAccessors
  def inverter
    @inverter
  end

  def mqtt_client
    @mqtt_client
  end
  # rubocop:enable Style/TrivialAccessors

  def publish(topic, payload)
    mqtt_client.publish(topic, payload)
  end

  def wait_for_mqtt(topic)
    message = with_retries { @messages.find { |h| h[:topic] == topic } }
    message&.fetch(:data)
  end

  def sqlite_inputs_table
    @sqlite_inputs_table ||= Sequel.sqlite('tmp/db/lxp.db')[:inputs]
  end
end
