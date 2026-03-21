use fsdr_blocks::agc::AgcBuilder;
use futuresdr::blocks::VectorSink;
use futuresdr::blocks::VectorSource;
use futuresdr::macros::connect;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Result;
use futuresdr::runtime::Runtime;

#[test]
fn agc_convergence_f32() -> Result<()> {
    let mut fg = Flowgraph::new();

    // Input signal with power 0.01 (amplitude 0.1)
    // Reference power 1.0
    let agc = AgcBuilder::<f32>::new()
        .adjustment_rate(0.1)
        .reference_power(1.0)
        .build();

    let orig: Vec<f32> = vec![0.1; 1000];
    let src = VectorSource::<f32>::new(orig.clone());
    let vect_sink = VectorSink::<f32>::new(1000);

    connect!(fg,
        src > agc > vect_sink;
    );
    Runtime::new().run(fg)?;

    let snk = vect_sink.get()?;
    let v = snk.items();

    // After some time, output amplitude should be close to 1.0 (power 1.0)
    let last_vals = &v[v.len() - 10..];
    for &val in last_vals {
        assert!(
            (val.abs() - 1.0).abs() < 0.1,
            "Value {} not converged to ~1.0",
            val
        );
    }

    Ok(())
}

#[test]
fn agc_squelch_f32() -> Result<()> {
    let mut fg = Flowgraph::new();

    let agc = AgcBuilder::<f32>::new()
        .squelch(0.5) // power threshold
        .build();

    // Input amplitude 0.1 => power 0.01 < 0.5 (squelched)
    // Input amplitude 1.0 => power 1.0 > 0.5 (not squelched)
    let orig: Vec<f32> = vec![0.1, 1.0];
    let src = VectorSource::<f32>::new(orig.clone());
    let vect_sink = VectorSink::<f32>::new(10);

    connect!(fg,
        src > agc > vect_sink;
    );
    Runtime::new().run(fg)?;

    let snk = vect_sink.get()?;
    let v = snk.items();

    assert_eq!(v[0], 0.0);
    assert!(v[1] != 0.0);

    Ok(())
}

#[test]
fn agc_gain_lock_f32() -> Result<()> {
    let mut fg = Flowgraph::new();

    let agc = AgcBuilder::<f32>::new()
        .gain_lock(true)
        .reference_power(1.0)
        .build();

    let orig: Vec<f32> = vec![0.1; 100];
    let src = VectorSource::<f32>::new(orig.clone());
    let vect_sink = VectorSink::<f32>::new(100);

    connect!(fg,
        src > agc > vect_sink;
    );
    Runtime::new().run(fg)?;

    let snk = vect_sink.get()?;
    let v = snk.items();

    // Gain is 1.0 by default, locked, so output should be equal to input
    for (&input, &output) in orig.iter().zip(v.iter()) {
        assert_eq!(input, output);
    }

    Ok(())
}
