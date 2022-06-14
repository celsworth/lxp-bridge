use crate::prelude::*;

pub fn u16ify(array: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([array[offset], array[offset + 1]])
}

pub fn le_i16_div100(input: &[u8]) -> nom::IResult<&[u8], f64> {
    let (input, num) = nom::number::complete::le_i16(input)?;
    Ok((input, num as f64 / 100.0))
}
pub fn le_u16_div10(input: &[u8]) -> nom::IResult<&[u8], f64> {
    let (input, num) = nom::number::complete::le_u16(input)?;
    Ok((input, num as f64 / 10.0))
}
pub fn le_u16_div100(input: &[u8]) -> nom::IResult<&[u8], f64> {
    let (input, num) = nom::number::complete::le_u16(input)?;
    Ok((input, num as f64 / 100.0))
}
pub fn le_u16_div1000(input: &[u8]) -> nom::IResult<&[u8], f64> {
    let (input, num) = nom::number::complete::le_u16(input)?;
    Ok((input, num as f64 / 1000.0))
}
pub fn le_u32_div10(input: &[u8]) -> nom::IResult<&[u8], f64> {
    let (input, num) = nom::number::complete::le_u32(input)?;
    Ok((input, num as f64 / 10.0))
}

pub fn current_time(input: &[u8]) -> nom::IResult<&[u8], UnixTime> {
    Ok((input, UnixTime::now()))
}
