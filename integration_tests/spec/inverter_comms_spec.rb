# frozen_string_literal: true

RSpec.describe 'inverter communications' do
  describe 'reading holding registers' do # {{{
    context 'reading register 12' do
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
  end # }}}

  describe 'reading input registers' do # {{{
    shared_examples_for 'processing input registers 0-39' do # {{{
      it 'publishes inputs/1' do
        payload = wait_for_mqtt('lxp/2222222222/inputs/1')

        expected = fixture('read_inputs_0_to_39_ok.decoded')
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
    end # }}}

    shared_examples_for 'processing input registers 40-79' do # {{{
      it 'publishes inputs/2' do
        payload = wait_for_mqtt('lxp/2222222222/inputs/2')

        expected = fixture('read_inputs_40_to_79_ok.decoded')
        expected['time'] = anything # changes in every run
        expect(payload).to include(expected)
      end

      it 'publishes individual input messages' do
        # publish_individual_input checks
        payload = wait_for_mqtt('lxp/2222222222/input/40')
        expect(payload).to eq 42_158

        payload = wait_for_mqtt('lxp/2222222222/input/59')
        expect(payload).to eq 1
      end
    end # }}}

    shared_examples_for 'processing input registers 80-119' do # {{{
      it 'publishes inputs/3' do
        payload = wait_for_mqtt('lxp/2222222222/inputs/3')

        expected = fixture('read_inputs_80_to_119_ok.decoded')
        expected['time'] = anything # changes in every run
        expect(payload).to include(expected)
      end

      it 'publishes individual input messages' do
        # publish_individual_input checks
        payload = wait_for_mqtt('lxp/2222222222/input/80')
        expect(payload).to eq 17

        payload = wait_for_mqtt('lxp/2222222222/input/90')
        expect(payload).to eq 0
      end
    end # }}}

    context 'reading input registers 0-39 via read/input/0' do # {{{
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
    end # }}}

    context 'reading input registers 0-39 via read/inputs/1' do # {{{
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
    end # }}}

    context 'receiving unprompted input registers 0-39' do # {{{
      before { inverter.write(fixture('read_inputs_0_to_39_ok.packet')) }

      it_behaves_like 'processing input registers 0-39'
    end # }}}

    context 'reading input registers 40-79 via read/input/40' do # {{{
      before do
        publish('lxp/cmd/all/read/input/40', '40')

        # check that inverter receives correct bytes
        expect(inverter.recv).to eq fixture('cmds/read_input_40')

        # send fake response from inverter
        response = fixture('read_inputs_40_to_79_ok.packet')
        inverter.write(response)
      end

      it_behaves_like 'processing input registers 40-79'

      it 'publishes result topic' do
        payload = wait_for_mqtt('lxp/result/2222222222/read/input/40')
        expect(payload).to eq 'OK'
      end
    end # }}}

    context 'receiving unprompted input registers 40-79' do # {{{
      before { inverter.write(fixture('read_inputs_40_to_79_ok.packet')) }

      it_behaves_like 'processing input registers 40-79'
    end # }}}

    context 'reading input registers 80-119 via read/input/80' do # {{{
      before do
        publish('lxp/cmd/all/read/input/80', '40')

        # check that inverter receives correct bytes
        expect(inverter.recv).to eq fixture('cmds/read_input_80')

        # send fake response from inverter
        response = fixture('read_inputs_80_to_119_ok.packet')
        inverter.write(response)
      end

      it_behaves_like 'processing input registers 80-119'

      it 'publishes result topic' do
        payload = wait_for_mqtt('lxp/result/2222222222/read/input/80')
        expect(payload).to eq 'OK'
      end
    end # }}}

    context 'receiving unprompted input registers 80-119' do # {{{
      before { inverter.write(fixture('read_inputs_80_to_119_ok.packet')) }

      it_behaves_like 'processing input registers 80-119'
    end # }}}
  end # }}}

  context 'receiving all unprompted input registers' do # {{{
    before { sleep 0.2 } # avoid test seeing a row write from a previous test

    subject do
      inverter.write(fixture('read_inputs_0_to_39_ok.packet'))
      inverter.write(fixture('read_inputs_40_to_79_ok.packet'))
      inverter.write(fixture('read_inputs_80_to_119_ok.packet'))
      # meh. be better to poll the table with a much shorter sleep.
      # better still to be told when the row appears but thats not gonna happen.
      sleep 0.5
    end

    it 'creates database row' do
      expect { subject }.to change { sqlite_inputs_table.count }.by(1)
    end
  end # }}}
end
