
#[cfg(test)]
mod tests {
    use std::hint::black_box;
    extern crate test;
    // use super::*;
    use test::Bencher;

    use futuresdr::blocks::Apply;
    use futuresdr::blocks::ApplyNM;

    use std::simd::{f32x8, f32x16};
    const N: usize = 2048;

    #[bench]
    fn bench_apply_vanilla_add_const(b: &mut Bencher) {
        let data = vec![1.0f32; N];
        let constant = black_box(3.0f32);
        let adder = Apply::new_typed(move |v: &f32| -> f32 { v + constant });
        let mut mocker = futuresdr::runtime::Mocker::new(adder);

        b.iter(|| {
            mocker.input::<f32>(0, black_box(data.to_owned()));
            mocker.init_output::<f32>(0, N);

            mocker.run();
            let _output = black_box(mocker.output::<f32>(0));
        });
    }

    #[bench]
    fn bench_applynm_16_add_const(b: &mut Bencher) {
        let data = vec![1.0f32; N];
        let constant = black_box(3.0f32);
        let adder = ApplyNM::<_,f32,f32,16,16>::new_typed(move |i: &[f32; 16], o: &mut [f32; 16]| {
            for (v, r) in i.iter().zip(o.iter_mut()) {
                *r = v + constant;
            }
        });
        let mut mocker = futuresdr::runtime::Mocker::new(adder);

        b.iter(|| {
            mocker.input::<f32>(0, black_box(data.to_owned()));
            mocker.init_output::<f32>(0, N);

            mocker.run();
            let _output = black_box(mocker.output::<f32>(0));
        });
    }
   
    #[bench]
    fn bench_applynm_simd_f32x16_add_const(b: &mut Bencher) {
        let data = vec![1.0f32; N];
        let constant = black_box(3.0f32);
        let constant= f32x16::from_array([constant; 16]); 
        let adder = ApplyNM::<_,f32,f32,16,16>::new_typed(move |i: &[f32; 16], o: &mut [f32; 16]| {
            let v = f32x16::from_slice(i);
            let r = v + constant;
            r.copy_to_slice(o);
        });
        let mut mocker = futuresdr::runtime::Mocker::new(adder);

        b.iter(|| {
            mocker.input::<f32>(0, black_box(data.to_owned()));
            mocker.init_output::<f32>(0, N);

            mocker.run();
            let _output = black_box(mocker.output::<f32>(0));
        });
    }

    #[bench]
    fn bench_applynm_simd_f32x8_add_const(b: &mut Bencher) {
        let data = vec![1.0f32; N];
        let constant = black_box(3.0f32);
        let constant= f32x8::from_array([constant; 8]); 
        let adder = ApplyNM::<_,f32,f32,8,8>::new_typed(move |i: &[f32; 8], o: &mut [f32; 8]| {
            let v = f32x8::from_slice(i);
            let r = v + constant;
            r.copy_to_slice(o);
        });
        let mut mocker = futuresdr::runtime::Mocker::new(adder);

        b.iter(|| {
            mocker.input::<f32>(0, black_box(data.to_owned()));
            mocker.init_output::<f32>(0, N);

            mocker.run();
            let _output = black_box(mocker.output::<f32>(0));
        });
    }
}