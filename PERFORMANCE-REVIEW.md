# PERFORMANCE REVIEW - fsdr-blocks

## Executive Summary
The current implementation of `fsdr-blocks` prioritized correctness and idiomatic Rust over raw DSP performance. While functional, the library suffers from several "performance killers" in hot loops and lacks modern SIMD optimizations found in established toolkits like `SatDump` or `csdr`.

## Findings

### 1. Hot Loop "Performance Killers"
- **AGC Block (`src/agc.rs`):** 
    - **Issue:** Uses `log10()` and `powi(2)` inside the per-sample loop. Transcendental functions are extremely expensive and stall the pipeline.
    - **Comparison:** `SatDump` and `csdr` use linear error-driven gain adjustments (e.g., `gain += rate * (reference - magnitude)`).
    - **Impact:** This is likely the single biggest bottleneck for any flowgraph using AGC.

### 2. Lack of SIMD / Autovectorization
- **Compiler Confirmation:** Analysis of the `cargo build --release` output with LLVM optimization remarks (`-C remark=all`) confirms that the core hot loops in `agc.rs`, `freq_shift.rs`, and `deinterleave.rs` completely **fail to auto-vectorize**.
- **Iterators vs. Slices:** While Rust iterators are often optimized well, explicit SIMD (via `std::simd` or `pulp`) or structure that aids autovectorization is missing.
- **Deinterleaving (`src/stream/deinterleave.rs`):** The current implementation uses a boolean toggle and two separate iterators. The compiler remarks indicate this loop is not vectorized due to the internal control flow (branching) and state toggling.
- **Frequency Shifting (`src/math/freq_shift.rs`):** The recurrence relation (`current_phasor *= rotation`) creates a loop-carried dependency that the compiler cannot safely vectorize under strict floating-point rules.
- **Conversion:** `type_converters.rs` performs scalar conversion of I/Q samples. Established projects use `volk` (e.g., `volk_16i_s32f_convert_32f`) to process these in bulk using AVX/NEON instructions.

### 3. NCO & Frequency Shifting
- **Fixed-Point Implementation:** FutureSDR uses a 32-bit fixed-point phase accumulator (`FixedPointPhase`) with a 1024-entry lookup table (`1 << 10`).
- **Linear Interpolation:** The NCO implementation uses first-order linear interpolation (`y = mx + b`) for both sine and cosine, which provides a good balance between accuracy and speed compared to raw `f32::sin/cos`.
- **Recurrence Error:** 
    - **Confirmed:** The `Complex32` kernel in `src/math/freq_shift.rs` uses a recurrence relation (`current_phasor *= rotation`) inside the hot loop to avoid two table lookups per sample.
    - **Impact:** While this is computationally efficient, floating-point rounding errors accumulate over the duration of the work call, causing the phasor magnitude and phase to drift away from the ideal value.
- **Mathematical Overhead:** Established projects like `csdr` often use `sincosf` (which map to a single hardware instruction on modern CPUs) or SIMD-vectorized lookup tables. The current scalar lookup + interpolation prevents effective autovectorization.

### 4. Memory Patterns
- **Buffer Management:** `FutureSDR` handles the buffers, but the way blocks access them (zip of iterators) can sometimes introduce bounds checks if the compiler cannot prove lengths are equal.
- **Branching:** Boolean flags inside loops (like in `Deinterleave`) prevent efficient pipelining.
- **NCO Step Overhead:** Calling `self.nco.step()` inside the `f32` loop involves non-trivial fixed-point arithmetic and wrapping that can hinder the compiler's ability to unroll or vectorize the multiplication.

## Comparison Table

| Feature | fsdr-blocks | SatDump | csdr |
| :--- | :--- | :--- | :--- |
| **AGC** | `log10` based (Slow) | Linear (Fast) | Alpha-Beta Filtered |
| **SIMD** | None (Compiler dependent) | Extensive (Volk) | Target Clones (FMV) |
| **Frequency Shift** | Recurrence | Volk / Lookup | Recurrence + FMV |
| **Converters** | Scalar | Volk | Scalar + Compiler Opt |

## Prioritized Next Steps

### Phase 1: Low-Hanging Fruit (Algorithmic)
1. **Refactor AGC:** Replace `log10` and `powi(2)` with magnitude-based or squared-magnitude-based linear adjustments.
2. **Deinterleave Optimization:** Process samples in pairs (chunks of 2) to eliminate the boolean branch and enable autovectorization.
3. **NCO Reset:** Periodically re-calculate the NCO phasor from `sin`/`cos` to prevent error accumulation in the recurrence relation.

### Phase 2: Platform-Agnostic Optimization
1. **Sincos:** Use `f32::sin_cos` to allow the compiler to use optimized platform-specific instructions.
2. **Loop Unrolling:** Manually unroll critical loops (like in `FreqShift`) to 4x or 8x to assist the optimizer.

### Phase 3: Hardware Acceleration (SIMD)
1. **Introduce SIMD:** Evaluate `pulp` or `std::simd` for cross-platform SIMD in `type_converters` and `math` blocks.
2. **Volk-like Dispatch:** Consider a "dispatch" pattern similar to `csdr`'s `target_clones` using the `multiversion` crate or similar Rust idioms.
