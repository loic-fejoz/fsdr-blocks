use crate::futuresdr::runtime::Block;
use crate::futuresdr::blocks::ApplyNM;

pub fn pack_8_in_1() -> Block {
    ApplyNM::<_, u8, u8, 8, 1>::new(move |v: &[u8], d: &mut [u8]| {
        d[0] = v
            .iter().rev()
            .enumerate()
            .map(|(i, u)| (*u) << i)
            .reduce(|a, b| a | b)
            .expect("guaranteee to not be empty due to ApplyNM");
    })
}