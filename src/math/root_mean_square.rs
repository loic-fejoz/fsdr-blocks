use crate::RecursiveBlock;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Block;
pub struct RootMeanSquareBuilder<A, const L: usize>
where
    A: Send + 'static + Copy,
{
    _item: std::marker::PhantomData<A>,
}

impl<A, const L: usize> RootMeanSquareBuilder<A, L>
where
    A: Send + 'static + Copy,
{
    pub fn new() -> Self {
        RootMeanSquareBuilder {
            _item: std::marker::PhantomData,
        }
    }
}

impl<A, const L: usize> Default for RootMeanSquareBuilder<A, L>
where
    A: Send + 'static + Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const L: usize> RootMeanSquareBuilder<f32, L> {
    pub fn build(self) -> Block {
        let mut y_n_1_pow2 = 0.0f32;
        // y_n = sqrt(y_n-1^2 + 1/L * (x_n^2 - x_n-L^2))
        RecursiveBlock::<f32, f32, _, L>::new(move |x_n: &f32, x_n_l_pow2: &f32| {
            let x_n_pow2 = x_n.powi(2);
            let y_n_pow2 = y_n_1_pow2 + (x_n_pow2 - x_n_l_pow2) / (L as f32);
            y_n_1_pow2 = y_n_pow2;
            let y_n = y_n_pow2.sqrt();
            (y_n, x_n_pow2)
        })
    }
}

impl<const L: usize> RootMeanSquareBuilder<Complex32, L> {
    pub fn build(self) -> Block {
        let mut y_n_1_pow2 = 0.0f32;
        // y_n = sqrt(y_n-1^2 + 1/L * (x_n^2 - x_n-L^2))
        RecursiveBlock::<Complex32, f32, _, L>::new(move |x_n: &Complex32, x_n_l_pow2: &f32| {
            let x_n_pow2 = x_n.norm_sqr();
            let y_n_pow2 = y_n_1_pow2 + (x_n_pow2 - x_n_l_pow2) / (L as f32);
            y_n_1_pow2 = y_n_pow2;
            let y_n = y_n_pow2.sqrt();
            (y_n, x_n_pow2)
        })
    }
}
