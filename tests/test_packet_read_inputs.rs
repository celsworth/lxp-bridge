mod common;
use common::*;

#[tokio::test]
async fn handles_missing_read_input() {
    let mut read_inputs = lxp::packet::ReadInputs::default();
    read_inputs.set_read_input_1(Factory::read_input_1());
    let ria = read_inputs.to_input_all();
    assert_eq!(ria, None);

    read_inputs.set_read_input_2(Factory::read_input_2());
    let ria = read_inputs.to_input_all();
    assert_eq!(ria.as_ref().unwrap().status, 16);
    assert_eq!(ria.unwrap().cycle_count, None);

    read_inputs.set_read_input_3(Factory::read_input_3());
    assert_eq!(read_inputs.to_input_all(), Some(Factory::read_input_all()));

    let mut read_inputs = lxp::packet::ReadInputs::default();
    read_inputs.set_read_input_3(Factory::read_input_3());
    assert_eq!(read_inputs.to_input_all(), None);
}
