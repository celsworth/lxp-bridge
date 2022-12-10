mod common;
use common::*;

#[tokio::test]
#[cfg_attr(not(feature = "mocks"), ignore)]
async fn handles_missing_read_input() {
    let mut read_inputs = lxp::packet::ReadInputs::default();
    read_inputs.set_read_input_1(Factory::read_input_1());
    assert_eq!(read_inputs.to_input_all(), None);

    read_inputs.set_read_input_2(Factory::read_input_2());
    assert_eq!(read_inputs.to_input_all(), None);

    read_inputs.set_read_input_3(Factory::read_input_3());
    assert_eq!(read_inputs.to_input_all(), Some(Factory::read_input_all()));

    let mut read_inputs = lxp::packet::ReadInputs::default();
    read_inputs.set_read_input_3(Factory::read_input_3());
    assert_eq!(read_inputs.to_input_all(), None);
}
