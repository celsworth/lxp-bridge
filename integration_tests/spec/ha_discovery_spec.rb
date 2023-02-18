# frozen_string_literal: true

RSpec.describe 'HomeAssistant discovery messages' do
  shared_examples 'has common payload' do
    it 'has common payload' do
      expect(payload)
        .to include 'availability' => { 'topic' => 'lxp/LWT' },
                    'device' => {
                      'manufacturer' => 'LuxPower',
                      'name' => 'lxp_2222222222',
                      'identifiers' => ['lxp_2222222222']
                    }
    end
  end

  context 'numbers' do
    let(:topic_prefix) { 'homeassistant/number/lxp_2222222222' }

    describe 'AcChargeEndSocLimit' do
      let(:payload) { wait_for_mqtt("#{topic_prefix}/AcChargeEndSocLimit/config") }

      it_behaves_like 'has common payload'

      it 'has expected payload' do
        expect(payload).to include 'name' => 'Charge From AC Upper Limit %',
                                   'state_topic' => 'lxp/2222222222/hold/161',
                                   'command_topic' => 'lxp/cmd/2222222222/set/hold/161',
                                   'value_template' => '{{ float(value) }}',
                                   'unique_id' => 'lxp_2222222222_number_AcChargeEndSocLimit',
                                   'min' => 0.0,
                                   'max' => 100.0,
                                   'step' => 1.0,
                                   'unit_of_measurement' => '%'
      end
    end

    describe 'EpsDischgCutoffSocEod' do
      let(:payload) { wait_for_mqtt("#{topic_prefix}/EpsDischgCutoffSocEod/config") }

      it_behaves_like 'has common payload'

      it 'has expected payload' do
        expect(payload).to include 'name' => 'Discharge Cutoff for EPS %',
                                   'state_topic' => 'lxp/2222222222/hold/125',
                                   'command_topic' => 'lxp/cmd/2222222222/set/hold/125',
                                   'value_template' => '{{ float(value) }}',
                                   'unique_id' => 'lxp_2222222222_number_EpsDischgCutoffSocEod',
                                   'min' => 0.0,
                                   'max' => 100.0,
                                   'step' => 1.0,
                                   'unit_of_measurement' => '%'
      end
    end
  end
end
