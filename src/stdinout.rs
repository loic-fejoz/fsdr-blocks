//! ## Blocks related to stdin/stdout serialization

use core::marker::PhantomData;
use futuresdr::blocks::Sink;
use futuresdr::num_complex::Complex32;
use std::io::Write;

#[derive(Clone, Copy)]
pub enum StdDirection {
    In,
    Out,
}

#[derive(Clone, Copy)]
pub enum BytesOrder {
    Native,
    BigEndian,
    LittleEndian,
}

/// Build blocks to serialize/deserialized stream from stdin/stdout.
/// It also takes care of endianness.
///
/// # Usage
///
/// Build a block that outputs a stream of u8 with native endianness to stdout:
/// ```
/// # use fsdr_blocks::stdinout::StdInOutBuilder;
/// let blk = StdInOutBuilder::<u8>::stdout().as_ne().build();
/// ```
///
/// Build a block that outputs a stream of `u8` with little endianness to stdout:
/// ```
/// # use fsdr_blocks::stdinout::StdInOutBuilder;
/// let blk = StdInOutBuilder::<u8>::stdout().as_le().build();
/// ```
pub struct StdInOutBuilder<A> {
    direction: StdDirection,
    marker_type: PhantomData<A>,
    bytes_order: BytesOrder,
}

impl<A> StdInOutBuilder<A> {
    pub fn stdin() -> StdInOutBuilder<A> {
        StdInOutBuilder::<A> {
            marker_type: PhantomData,
            direction: StdDirection::In,
            bytes_order: BytesOrder::Native,
        }
    }

    pub fn stdout() -> StdInOutBuilder<A> {
        StdInOutBuilder::<A> {
            marker_type: PhantomData,
            direction: StdDirection::Out,
            bytes_order: BytesOrder::Native,
        }
    }

    pub fn as_ne(self) -> StdInOutBuilder<A> {
        StdInOutBuilder::<A> {
            bytes_order: BytesOrder::Native,
            ..self
        }
    }

    pub fn as_le(self) -> StdInOutBuilder<A> {
        StdInOutBuilder::<A> {
            bytes_order: BytesOrder::LittleEndian,
            ..self
        }
    }

    pub fn as_be(self) -> StdInOutBuilder<A> {
        StdInOutBuilder::<A> {
            bytes_order: BytesOrder::BigEndian,
            ..self
        }
    }
}

pub trait ToEndianBytes {
    fn write_to<W: Write>(&self, w: &mut W, order: BytesOrder) -> std::io::Result<()>;
}

impl ToEndianBytes for u8 {
    #[inline]
    fn write_to<W: Write>(&self, w: &mut W, _order: BytesOrder) -> std::io::Result<()> {
        w.write_all(&[*self])
    }
}

impl ToEndianBytes for i16 {
    #[inline]
    fn write_to<W: Write>(&self, w: &mut W, order: BytesOrder) -> std::io::Result<()> {
        match order {
            BytesOrder::Native => w.write_all(&self.to_ne_bytes()),
            BytesOrder::LittleEndian => w.write_all(&self.to_le_bytes()),
            BytesOrder::BigEndian => w.write_all(&self.to_be_bytes()),
        }
    }
}

impl ToEndianBytes for f32 {
    #[inline]
    fn write_to<W: Write>(&self, w: &mut W, order: BytesOrder) -> std::io::Result<()> {
        match order {
            BytesOrder::Native => w.write_all(&self.to_ne_bytes()),
            BytesOrder::LittleEndian => w.write_all(&self.to_le_bytes()),
            BytesOrder::BigEndian => w.write_all(&self.to_be_bytes()),
        }
    }
}

impl ToEndianBytes for Complex32 {
    #[inline]
    fn write_to<W: Write>(&self, w: &mut W, order: BytesOrder) -> std::io::Result<()> {
        match order {
            BytesOrder::Native => {
                w.write_all(&self.re.to_ne_bytes())?;
                w.write_all(&self.im.to_ne_bytes())
            }
            BytesOrder::LittleEndian => {
                w.write_all(&self.re.to_le_bytes())?;
                w.write_all(&self.im.to_le_bytes())
            }
            BytesOrder::BigEndian => {
                w.write_all(&self.re.to_be_bytes())?;
                w.write_all(&self.im.to_be_bytes())
            }
        }
    }
}

impl<A: ToEndianBytes + Send + Sync + Default + Clone + std::fmt::Debug + 'static>
    StdInOutBuilder<A>
{
    pub fn build(self) -> Sink<impl FnMut(&A) + Send + 'static, A> {
        match self.direction {
            StdDirection::Out => {
                let mut stdout = std::io::BufWriter::new(std::io::stdout());
                let bytes_order = self.bytes_order;
                Sink::new(move |f: &A| {
                    if let Err(e) = f.write_to(&mut stdout, bytes_order)
                        && e.kind() != std::io::ErrorKind::BrokenPipe
                    {
                        eprintln!("StdInOut: write error: {e}");
                    }
                })
            }
            StdDirection::In => todo!("stdin not yet implemented"),
        }
    }
}
