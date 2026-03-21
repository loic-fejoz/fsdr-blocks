use futuresdr::num_complex::{Complex32, ComplexFloat};
use futuresdr::prelude::*;

#[cfg(feature = "simd")]
use core::simd::Select;
#[cfg(feature = "simd")]
use core::simd::num::SimdFloat;
#[cfg(feature = "simd")]
use core::simd::prelude::*;

/// Automatic Gain Control Block
#[derive(Block)]
#[message_inputs(auto_lock, gain_lock, max_gain, adjustment_rate, reference_power)]
pub struct Agc<
    T: Send + Sync + ComplexFloat<Real = f32> + Default + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = T> = DefaultCpuReader<T>,
    O: CpuBufferWriter<Item = T> = DefaultCpuWriter<T>,
> {
    #[input]
    input: I,
    #[output]
    output: O,
    /// Minimum value that has to be reached in order for AGC to start adjusting gain.
    squelch: f32,
    /// maximum gain value
    max_gain: f32,
    /// initial gain value.
    gain: f32,
    /// reference value to adjust signal power to.
    reference_power: f32,
    /// the update rate of the loop.
    adjustment_rate: f32,
    /// Set when gain should not be adjusted anymore, but rather be locked to the current value
    gain_lock: bool,
    /// Set when gain should be automatically locked, when reference power is reached.
    auto_lock: bool,
}

impl<T, I, O> Agc<T, I, O>
where
    T: Send + Sync + ComplexFloat<Real = f32> + Default + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = T>,
    O: CpuBufferWriter<Item = T>,
{
    /// Create AGC Block
    pub fn new(
        squelch: f32,
        max_gain: f32,
        gain: f32,
        adjustment_rate: f32,
        reference_power: f32,
        gain_lock: bool,
        auto_lock: bool,
    ) -> Self {
        assert!(max_gain >= 0.0);
        assert!(squelch >= 0.0);

        Agc {
            input: I::default(),
            output: O::default(),
            squelch,
            max_gain,
            gain,
            reference_power,
            adjustment_rate,
            gain_lock,
            auto_lock,
        }
    }

    async fn auto_lock(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        if let Pmt::Bool(l) = p {
            self.auto_lock = l;
            Ok(Pmt::Ok)
        } else {
            Ok(Pmt::InvalidValue)
        }
    }

    async fn gain_lock(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        if let Pmt::Bool(l) = p {
            self.gain_lock = l;
            Ok(Pmt::Ok)
        } else {
            Ok(Pmt::InvalidValue)
        }
    }

    async fn max_gain(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        if let Pmt::F32(m) = p {
            self.max_gain = m;
            Ok(Pmt::Ok)
        } else {
            Ok(Pmt::InvalidValue)
        }
    }

    async fn adjustment_rate(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        if let Pmt::F32(r) = p {
            self.adjustment_rate = r;
            Ok(Pmt::Ok)
        } else {
            Ok(Pmt::InvalidValue)
        }
    }

    async fn reference_power(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        if let Pmt::F32(r) = p {
            self.reference_power = r;
            Ok(Pmt::Ok)
        } else {
            Ok(Pmt::InvalidValue)
        }
    }
}

pub trait AgcSupported: Copy {
    fn process(
        gain: &mut f32,
        gain_lock: &mut bool,
        squelch: f32,
        max_gain: f32,
        adjustment_rate: f32,
        reference_power: f32,
        auto_lock: bool,
        input: &[Self],
        output: &mut [Self],
    );
}

fn agc_scalar_logic<T>(
    gain: &mut f32,
    gain_lock: &mut bool,
    squelch: f32,
    _max_gain: f32,
    adjustment_rate: f32,
    reference_power: f32,
    auto_lock: bool,
    input: &[T],
    output: &mut [T],
) where
    T: ComplexFloat<Real = f32> + Copy,
{
    let n = input.len().min(output.len());
    let mut g = *gain;
    let mut gl = *gain_lock;

    for (src, dst) in input[..n].iter().zip(output[..n].iter_mut()) {
        let input_power = src.abs().powi(2);
        if input_power > squelch {
            let out = (*src) * T::from(g).unwrap();
            let output_power = out.abs().powi(2);

            if auto_lock {
                if input_power > reference_power {
                    if output_power < reference_power {
                        gl = true;
                    }
                } else if output_power > reference_power {
                    gl = true;
                }
            }

            if !gl {
                let dynamic_adjustment_rate = if adjustment_rate > 0.0 {
                    adjustment_rate
                } else {
                    0.0001
                };
                // Linear error instead of log10 for performance
                // error = (ref - out_power) / ref
                let error = (reference_power - output_power) / reference_power;
                g += error * dynamic_adjustment_rate * g;
                g = g.max(0.0);
            }
            *dst = out;
        } else {
            *dst = T::from(0.0).unwrap();
        }
    }
    *gain = g;
    *gain_lock = gl;
}

#[cfg(feature = "simd")]
impl<T> AgcSupported for T
where
    T: ComplexFloat<Real = f32> + Copy + Send + Sync + 'static,
{
    default fn process(
        gain: &mut f32,
        gain_lock: &mut bool,
        squelch: f32,
        max_gain: f32,
        adjustment_rate: f32,
        reference_power: f32,
        auto_lock: bool,
        input: &[Self],
        output: &mut [Self],
    ) {
        agc_scalar_logic(
            gain,
            gain_lock,
            squelch,
            max_gain,
            adjustment_rate,
            reference_power,
            auto_lock,
            input,
            output,
        );
    }
}

#[cfg(not(feature = "simd"))]
impl<T> AgcSupported for T
where
    T: ComplexFloat<Real = f32> + Copy + Send + Sync + 'static,
{
    fn process(
        gain: &mut f32,
        gain_lock: &mut bool,
        squelch: f32,
        max_gain: f32,
        adjustment_rate: f32,
        reference_power: f32,
        auto_lock: bool,
        input: &[Self],
        output: &mut [Self],
    ) {
        agc_scalar_logic(
            gain,
            gain_lock,
            squelch,
            max_gain,
            adjustment_rate,
            reference_power,
            auto_lock,
            input,
            output,
        );
    }
}

#[cfg(feature = "simd")]
impl AgcSupported for f32 {
    fn process(
        gain: &mut f32,
        gain_lock: &mut bool,
        squelch: f32,
        _max_gain: f32,
        adjustment_rate: f32,
        reference_power: f32,
        auto_lock: bool,
        input: &[Self],
        output: &mut [Self],
    ) {
        let n = input.len().min(output.len());
        const LANES: usize = 8;
        let n_simd = n / LANES;

        let v_squelch = f32x8::splat(squelch);
        let adj_rate = if adjustment_rate > 0.0 {
            adjustment_rate
        } else {
            0.0001
        };

        for i in 0..n_simd {
            let v_in = f32x8::from_slice(&input[i * LANES..]);
            let v_gain = f32x8::splat(*gain);
            let v_out = v_in * v_gain;

            let v_in_power = v_in * v_in;
            let v_out_power = v_out * v_out;

            let mask = v_in_power.simd_gt(v_squelch);
            let v_final_out = mask.select(v_out, f32x8::splat(0.0));
            v_final_out.copy_to_slice(&mut output[i * LANES..]);

            if !*gain_lock {
                // Approximate average error for the block to update gain
                // This is a trade-off: updating gain once per SIMD block
                let avg_out_power = v_out_power.reduce_sum() / LANES as f32;
                let error = (reference_power - avg_out_power) / reference_power;
                *gain += error * adj_rate * (*gain);
                *gain = gain.max(0.0);

                if auto_lock {
                    // Simplified auto-lock: check if average power is close enough
                    if (avg_out_power - reference_power).abs() < 0.01 * reference_power {
                        *gain_lock = true;
                    }
                }
            }
        }

        // Tail
        let tail_start = n_simd * LANES;
        if tail_start < n {
            agc_scalar_logic(
                gain,
                gain_lock,
                squelch,
                _max_gain,
                adjustment_rate,
                reference_power,
                auto_lock,
                &input[tail_start..n],
                &mut output[tail_start..n],
            );
        }
    }
}

#[cfg(feature = "simd")]
impl AgcSupported for Complex32 {
    fn process(
        gain: &mut f32,
        gain_lock: &mut bool,
        squelch: f32,
        _max_gain: f32,
        adjustment_rate: f32,
        reference_power: f32,
        auto_lock: bool,
        input: &[Self],
        output: &mut [Self],
    ) {
        let n = input.len().min(output.len());
        const LANES: usize = 8;
        let n_simd = n / LANES;

        let v_squelch = f32x8::splat(squelch);
        let adj_rate = if adjustment_rate > 0.0 {
            adjustment_rate
        } else {
            0.0001
        };

        let i_f32 = unsafe { core::slice::from_raw_parts(input.as_ptr() as *const f32, n * 2) };
        let o_f32 =
            unsafe { core::slice::from_raw_parts_mut(output.as_mut_ptr() as *mut f32, n * 2) };

        for i in 0..n_simd {
            let v0 = f32x8::from_slice(&i_f32[i * LANES * 2..]);
            let v1 = f32x8::from_slice(&i_f32[i * LANES * 2 + 8..]);
            let (v_re, v_im) = v0.deinterleave(v1);

            let v_gain = f32x8::splat(*gain);
            let v_out_re = v_re * v_gain;
            let v_out_im = v_im * v_gain;

            let v_in_power = v_re * v_re + v_im * v_im;
            let v_out_power = v_out_re * v_out_re + v_out_im * v_out_im;

            let mask = v_in_power.simd_gt(v_squelch);
            let v_final_re = mask.select(v_out_re, f32x8::splat(0.0));
            let v_final_im = mask.select(v_out_im, f32x8::splat(0.0));

            let (o0, o1) = v_final_re.interleave(v_final_im);
            o0.copy_to_slice(&mut o_f32[i * LANES * 2..]);
            o1.copy_to_slice(&mut o_f32[i * LANES * 2 + 8..]);

            if !*gain_lock {
                let avg_out_power = v_out_power.reduce_sum() / LANES as f32;
                let error = (reference_power - avg_out_power) / reference_power;
                *gain += error * adj_rate * (*gain);
                *gain = gain.max(0.0);

                if auto_lock {
                    if (avg_out_power - reference_power).abs() < 0.01 * reference_power {
                        *gain_lock = true;
                    }
                }
            }
        }

        let tail_start = n_simd * LANES;
        if tail_start < n {
            agc_scalar_logic(
                gain,
                gain_lock,
                squelch,
                _max_gain,
                adjustment_rate,
                reference_power,
                auto_lock,
                &input[tail_start..n],
                &mut output[tail_start..n],
            );
        }
    }
}

#[doc(hidden)]
impl<T, I, O> Kernel for Agc<T, I, O>
where
    T: Send
        + Sync
        + ComplexFloat<Real = f32>
        + Default
        + std::fmt::Debug
        + Copy
        + 'static
        + AgcSupported,
    I: CpuBufferReader<Item = T>,
    O: CpuBufferWriter<Item = T>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let m = {
            let i = self.input.slice();
            let o = self.output.slice();

            let m = std::cmp::min(i.len(), o.len());
            if m > 0 {
                T::process(
                    &mut self.gain,
                    &mut self.gain_lock,
                    self.squelch,
                    self.max_gain,
                    self.adjustment_rate,
                    self.reference_power,
                    self.auto_lock,
                    &i[..m],
                    &mut o[..m],
                );
            }
            m
        };

        if m > 0 {
            self.input.consume(m);
            self.output.produce(m);
        }

        if self.input.finished() && self.input.slice().is_empty() {
            io.finished = true;
        }

        Ok(())
    }
}

/// Builder for [`Agc`] block
pub struct AgcBuilder<T> {
    squelch: f32,
    max_gain: f32,
    gain: f32,
    adjustment_rate: f32,
    reference_power: f32,
    gain_lock: bool,
    auto_lock: bool,
    _type: std::marker::PhantomData<T>,
}

impl<T> AgcBuilder<T>
where
    T: Send
        + Sync
        + ComplexFloat<Real = f32>
        + Default
        + std::fmt::Debug
        + Copy
        + 'static
        + AgcSupported,
{
    pub fn new() -> AgcBuilder<T> {
        AgcBuilder {
            squelch: 0.0,
            max_gain: 65536.0,
            gain: 1.0,
            adjustment_rate: 0.0001,
            reference_power: 1.0,
            gain_lock: false,
            auto_lock: false,
            _type: std::marker::PhantomData,
        }
    }

    pub fn squelch(mut self, squelch: f32) -> AgcBuilder<T> {
        self.squelch = squelch;
        self
    }

    pub fn max_gain(mut self, max_gain: f32) -> AgcBuilder<T> {
        self.max_gain = max_gain;
        self
    }

    pub fn adjustment_rate(mut self, adjustment_rate: f32) -> AgcBuilder<T> {
        self.adjustment_rate = adjustment_rate;
        self
    }

    pub fn reference_power(mut self, reference_power: f32) -> AgcBuilder<T> {
        self.reference_power = reference_power;
        self
    }

    /// Fix gain setting, disabling AGC
    pub fn gain_lock(mut self, gain_lock: bool) -> AgcBuilder<T> {
        self.gain_lock = gain_lock;
        self
    }

    pub fn auto_lock(mut self, auto_lock: bool) -> AgcBuilder<T> {
        self.auto_lock = auto_lock;
        self
    }

    /// Create [`Agc`] block
    pub fn build(self) -> Agc<T> {
        Agc::<T>::new(
            self.squelch,
            self.max_gain,
            self.gain,
            self.adjustment_rate,
            self.reference_power,
            self.gain_lock,
            self.auto_lock,
        )
    }
}

impl<T> Default for AgcBuilder<T>
where
    T: Send
        + Sync
        + ComplexFloat<Real = f32>
        + Default
        + std::fmt::Debug
        + Copy
        + 'static
        + AgcSupported,
{
    fn default() -> Self {
        Self::new()
    }
}
