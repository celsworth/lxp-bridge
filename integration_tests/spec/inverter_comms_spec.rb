# frozen_string_literal: true

RSpec.describe 'inverter communications' do
  context 'reading holding register 12' do
    before { publish('lxp/cmd/all/read/hold/12', '') }

    it 'publishes correct messages' do
      # check that inverter receives correct bytes
      expect(inverter.recv).to eq fixture('cmds/read_hold_12')

      # send fake response from inverter
      inverter.write(fixture('read_hold_12_ok.packet'))

      # 30 + (40 << 8) = 10270
      payload = wait_for_mqtt('lxp/2222222222/hold/12')
      expect(payload).to eq 10_270

      payload = wait_for_mqtt('lxp/result/2222222222/read/hold/12')
      expect(payload).to eq 'OK'
    end
  end

  shared_examples_for 'processing input registers 0-39' do
    it 'publishes inputs/1' do
      payload = wait_for_mqtt('lxp/2222222222/inputs/1')

      expected = fixture('read_inputs_0_to_39_ok.decoded')['data']
      expected['time'] = anything # changes in every run
      expect(payload).to include(expected)
    end

    it 'publishes individual input messages' do
      # publish_individual_input checks
      payload = wait_for_mqtt('lxp/2222222222/input/0')
      expect(payload).to eq 16

      payload = wait_for_mqtt('lxp/2222222222/input/15')
      expect(payload).to eq 5004
    end
  end

  context 'reading input registers 0-39 via read/input/0' do
    before do
      publish('lxp/cmd/all/read/input/0', '40')

      # check that inverter receives correct bytes
      expect(inverter.recv).to eq fixture('cmds/read_input_0')

      # send fake response from inverter
      response = fixture('read_inputs_0_to_39_ok.packet')
      inverter.write(response)
    end

    it_behaves_like 'processing input registers 0-39'

    it 'publishes result topic' do
      payload = wait_for_mqtt('lxp/result/2222222222/read/input/0')
      expect(payload).to eq 'OK'
    end
  end

  context 'reading input registers 0-39 via read/inputs/1' do
    before do
      publish('lxp/cmd/all/read/inputs/1', '')

      # check that inverter receives correct bytes
      expect(inverter.recv).to eq fixture('cmds/read_input_0')

      # send fake response from inverter
      response = fixture('read_inputs_0_to_39_ok.packet')
      inverter.write(response)
    end

    it_behaves_like 'processing input registers 0-39'

    it 'publishes result topic' do
      payload = wait_for_mqtt('lxp/result/2222222222/read/inputs/1')
      expect(payload).to eq 'OK'
    end
  end

  context 'receiving unprompted input registers 0-39' do
    before { inverter.write(fixture('read_inputs_0_to_39_ok.packet')) }

    it_behaves_like 'processing input registers 0-39'
  end
end
