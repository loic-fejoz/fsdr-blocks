use fsdr_blocks::math::RootMeanSquareBuilder;
use futuresdr::anyhow::Result;
use futuresdr::async_io::block_on;
use futuresdr::blocks::signal_source::SignalSourceBuilder;
use futuresdr::blocks::FiniteSource;
use futuresdr::blocks::Sink;
use futuresdr::macros::connect;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;
use std::iter;
use std::sync::Arc;
use std::sync::Mutex;

#[test]
fn rms_dc_f32() -> Result<()> {
    let mut fg = Flowgraph::new();

    const A: f32 = 4.0f32;

    let mut fours = iter::repeat(A).take(100);
    let src = FiniteSource::new(move || fours.next());
    let rms = RootMeanSquareBuilder::<f32, 10>::new().build();

    // After a while, x should be equal to A
    let mut counter: usize = 0;
    let sink = Sink::new(move |x: &f32| {
        counter += 1;
        if counter > 10 {
            assert!((x - A).abs() < 0.0000001);
        }
    });

    connect!(fg,
        src > rms > sink;
    );
    fg = Runtime::new().run(fg)?;

    Ok(())
}

#[test]
fn rms_sine_f32() -> Result<()> {
    let mut fg = Flowgraph::new();

    const A: f32 = 4.0f32;

    const FREQ: f32 = 50.0;
    const SAMPLE_RATE: f32 = 48_000.0;
    const WINDOW_SIZE: usize = (SAMPLE_RATE / FREQ) as usize;
    let src = SignalSourceBuilder::<f32>::sin(FREQ, SAMPLE_RATE)
        .amplitude(A)
        .build();
    let rms = RootMeanSquareBuilder::<f32, WINDOW_SIZE>::new().build();

    // After a while, x should be equal to A / sqrt(2)
    let expected = A / 2.0f32.sqrt();
    let mut counter: usize = 0;
    let tested = Arc::new(Mutex::new(false));
    let tested_for_check = Arc::clone(&tested);
    let tested = Arc::clone(&tested);
    let sink = Sink::new(move |x: &f32| {
        //println!("{x}");
        counter += 1;
        if counter > WINDOW_SIZE {
            let mut tested = tested_for_check.lock().unwrap();
            *tested = true;
            assert!((x - expected).abs() < 0.01);
        }
    });

    connect!(fg,
        src > rms > sink;
    );

    let rt = Runtime::new();
    let (fg, mut handle) = block_on(rt.start(fg));
    block_on(async move {
        futuresdr::async_io::Timer::after(std::time::Duration::from_secs(1)).await;
        handle.terminate().await.unwrap();
        let _ = fg.await;
    });
    let tested = tested.lock().unwrap();
    assert_eq!(true, *tested); // Ensure we run sufficiently enough to do some actual test

    Ok(())
}

#[test]
fn rms_square_f32() -> Result<()> {
    let mut fg = Flowgraph::new();

    const A: f32 = 4.0f32;

    const FREQ: f32 = 50.0;
    const SAMPLE_RATE: f32 = 48_000.0;
    const WINDOW_SIZE: usize = (SAMPLE_RATE / FREQ) as usize;
    let src = SignalSourceBuilder::<f32>::square(FREQ, SAMPLE_RATE)
        .amplitude(2.0 * A)
        .offset(-A)
        .build();
    let rms = RootMeanSquareBuilder::<f32, WINDOW_SIZE>::new().build();

    // After a while, x should be equal to A
    let expected = A;
    let mut counter: usize = 0;
    let tested = Arc::new(Mutex::new(false));
    let tested_for_check = Arc::clone(&tested);
    let tested = Arc::clone(&tested);
    let sink = Sink::new(move |x: &f32| {
        //println!("{x}");
        counter += 1;
        if counter > 3 * WINDOW_SIZE {
            let mut tested = tested_for_check.lock().unwrap();
            *tested = true;
            assert!((x - expected).abs() < 0.01);
        }
    });

    connect!(fg,
        src > rms > sink;
    );

    let rt = Runtime::new();
    let (fg, mut handle) = block_on(rt.start(fg));
    block_on(async move {
        futuresdr::async_io::Timer::after(std::time::Duration::from_secs(1)).await;
        handle.terminate().await.unwrap();
        let _ = fg.await;
    });
    let tested = tested.lock().unwrap();
    assert_eq!(true, *tested); // Ensure we run sufficiently enough to do some actual test

    Ok(())
}

#[test]
fn rms_sine_c32() -> Result<()> {
    let mut fg = Flowgraph::new();

    const A: f32 = 4.0f32;

    const FREQ: f32 = 50.0;
    const SAMPLE_RATE: f32 = 48_000.0;
    const WINDOW_SIZE: usize = (SAMPLE_RATE / FREQ) as usize;
    let src = SignalSourceBuilder::<Complex32>::cos(FREQ, SAMPLE_RATE)
        .amplitude(Complex32::new(A, 0.0))
        .build();
    let rms = RootMeanSquareBuilder::<Complex32, WINDOW_SIZE>::new().build();

    // After a while, x should be equal to A
    let expected = A;
    let mut counter: usize = 0;
    let tested = Arc::new(Mutex::new(false));
    let tested_for_check = Arc::clone(&tested);
    let tested = Arc::clone(&tested);
    let sink = Sink::new(move |x: &f32| {
        //println!("{x}");
        counter += 1;
        if counter > WINDOW_SIZE {
            let mut tested = tested_for_check.lock().unwrap();
            *tested = true;
            assert!((x - expected).abs() < 0.01);
        }
    });

    connect!(fg,
        src > rms > sink;
    );

    let rt = Runtime::new();
    let (fg, mut handle) = block_on(rt.start(fg));
    block_on(async move {
        futuresdr::async_io::Timer::after(std::time::Duration::from_secs(1)).await;
        handle.terminate().await.unwrap();
        let _ = fg.await;
    });
    let tested = tested.lock().unwrap();
    assert_eq!(true, *tested); // Ensure we run sufficiently enough to do some actual test

    Ok(())
}
