use futuresdr::anyhow::Result;

use futuresdr::runtime::Block;
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;
pub struct RecursiveBlock<A, B, F, const N: usize>
where
    A: Send + 'static,
    B: Send + 'static,
    F: FnMut(&A, &B) -> (B, B) + Send + 'static,
{
    f: F,
    _input_item: std::marker::PhantomData<A>,
    _output_item: std::marker::PhantomData<B>,
    buffer: [B; N],
    current_i: usize,
}

impl<A, B, F, const N: usize> RecursiveBlock<A, B, F, N>
where
    A: Send + 'static,
    B: Send + 'static + Default + Copy,
    F: FnMut(&A, &B) -> (B, B) + Send + 'static,
{
    pub fn new(f: F) -> Block {
        Block::new(
            BlockMetaBuilder::new("RecursiveBlock").build(),
            StreamIoBuilder::new()
                .add_input::<A>("in")
                .add_output::<B>("out")
                .build(),
            MessageIoBuilder::<Self>::new().build(),
            RecursiveBlock {
                f,
                _input_item: std::marker::PhantomData,
                _output_item: std::marker::PhantomData,
                buffer: [B::default(); N],
                current_i: 0,
            },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl<A, B, F, const N: usize> Kernel for RecursiveBlock<A, B, F, N>
where
    F: FnMut(&A, &B) -> (B, B) + Send + 'static,
    A: Send + 'static,
    B: Send + 'static,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i0 = sio.input(0).slice::<A>();
        let o0: &mut [B] = sio.output(0).slice::<B>();

        let m = std::cmp::min(i0.len(), o0.len());
        let remaining = N - self.current_i;
        let m = std::cmp::min(m, remaining);
        if m > 0 {
            let i1 = self.buffer[self.current_i..].iter_mut();
            for ((x_in, x_prev), y_output) in i0.iter().zip(i1).zip(o0.iter_mut()) {
                (*y_output, *x_prev) = (self.f)(x_in, x_prev);
            }
            self.current_i = (self.current_i + m - 1) % (N - 1);

            sio.input(0).consume(m);
            sio.output(0).produce(m);
        }

        if sio.input(0).finished() && m == i0.len() {
            io.finished = true;
        }

        Ok(())
    }
}
