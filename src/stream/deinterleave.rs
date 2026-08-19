use futuresdr::runtime::dev::prelude::*;

/// This blocks deinterleave a unique stream into two separate stream.
/// Typically used to deinterleave iq stream into of stream for `i` and one for `q`.
///
/// # Usage
/// ```
/// use fsdr_blocks::stream::Deinterleave;
/// let blk = Deinterleave::<f32>::new();
/// ```
#[derive(Block)]
pub struct Deinterleave<
    A: Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = A> = DefaultCpuReader<A>,
    O0: CpuBufferWriter<Item = A> = DefaultCpuWriter<A>,
    O1: CpuBufferWriter<Item = A> = DefaultCpuWriter<A>,
> {
    #[input]
    input: I,
    #[output]
    out0: O0,
    #[output]
    out1: O1,
    first: bool,
}

impl<A, I, O0, O1> Deinterleave<A, I, O0, O1>
where
    A: Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = A>,
    O0: CpuBufferWriter<Item = A>,
    O1: CpuBufferWriter<Item = A>,
{
    pub fn new() -> Self {
        Self {
            input: I::default(),
            out0: O0::default(),
            out1: O1::default(),
            first: true,
        }
    }
}

impl<A, I, O0, O1> Default for Deinterleave<A, I, O0, O1>
where
    A: Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = A>,
    O0: CpuBufferWriter<Item = A>,
    O1: CpuBufferWriter<Item = A>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<A, I, O0, O1> Kernel for Deinterleave<A, I, O0, O1>
where
    A: Send + Sync + Default + Copy + std::fmt::Debug + 'static,
    I: CpuBufferReader<Item = A>,
    O0: CpuBufferWriter<Item = A>,
    O1: CpuBufferWriter<Item = A>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let (m, m0, m1) = {
            let i0 = self.input.slice();
            let o0 = self.out0.slice();
            let o1 = self.out1.slice();

            let mut m0 = 0;
            let mut m1 = 0;

            let mut it0 = o0.iter_mut();
            let mut it1 = o1.iter_mut();

            for x in i0.iter() {
                if self.first {
                    if let Some(d) = it0.next() {
                        *d = *x;
                        m0 += 1;
                    } else {
                        break;
                    }
                } else {
                    if let Some(d) = it1.next() {
                        *d = *x;
                        m1 += 1;
                    } else {
                        break;
                    }
                }
                self.first = !self.first;
            }
            (m0 + m1, m0, m1)
        };

        self.input.consume(m);
        self.out0.produce(m0);
        self.out1.produce(m1);

        if self.input.finished() && self.input.slice().is_empty() {
            io.finished = true;
        }

        Ok(())
    }
}
