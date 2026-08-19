use futuresdr::blocks::signal_source::FixedPointPhase;
use futuresdr::blocks::signal_source::NCO;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;

/// This blocks shift the signal in the frequency domain based on the [`NCO`] implementation.
/// Currently implemented only for float and [`Complex32`]
///
/// # Usage
///
/// ```
/// # use futuresdr::num_complex::Complex32;
/// # use fsdr_blocks::math::FrequencyShifter;
/// # let freq = 2_000;
/// # let sample_rate = 48_000;
/// let blk = FrequencyShifter::<Complex32>::new(freq as f32, sample_rate as f32);
/// ```
#[derive(Block)]
pub struct FrequencyShifter<
    A: Send + Sync + Default + Clone + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = A> = DefaultCpuReader<A>,
    O: CpuBufferWriter<Item = A> = DefaultCpuWriter<A>,
> {
    #[input]
    input: I,
    #[output]
    output: O,
    nco: NCO,
    phase_inc: FixedPointPhase,
}

impl<A, I, O> FrequencyShifter<A, I, O>
where
    A: Send + Sync + Default + Clone + std::fmt::Debug + 'static + Copy,
    I: CpuBufferReader<Item = A>,
    O: CpuBufferWriter<Item = A>,
{
    /// Create FrequencyShifter block
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        let phase_inc = 2.0 * core::f32::consts::PI * frequency / sample_rate;
        let nco = NCO::new(0.0f32, phase_inc);
        Self {
            input: I::default(),
            output: O::default(),
            nco,
            phase_inc: FixedPointPhase::new(phase_inc),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for FrequencyShifter<f32, I, O>
where
    I: CpuBufferReader<Item = f32>,
    O: CpuBufferWriter<Item = f32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let m = {
            let i = self.input.slice();
            let o = self.output.slice();

            let m = std::cmp::min(i.len(), o.len());
            if m > 0 {
                for (v, r) in i[..m].iter().zip(o[..m].iter_mut()) {
                    *r = (*v) * self.nco.phase.cos();
                    self.nco.step();
                }
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

#[doc(hidden)]
impl<I, O> Kernel for FrequencyShifter<Complex32, I, O>
where
    I: CpuBufferReader<Item = Complex32>,
    O: CpuBufferWriter<Item = Complex32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let m = {
            let i = self.input.slice();
            let o = self.output.slice();

            let m = std::cmp::min(i.len(), o.len());
            if m > 0 {
                let rotation = Complex32::new(self.phase_inc.cos(), self.phase_inc.sin());
                let mut current_phasor = Complex32::new(self.nco.phase.cos(), self.nco.phase.sin());
                for (v, r) in i[..m].iter().zip(o[..m].iter_mut()) {
                    *r = (*v) * current_phasor;
                    current_phasor *= rotation;
                }
                self.nco.steps(m as i32);
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
