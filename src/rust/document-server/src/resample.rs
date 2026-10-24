//! ImageSharp 2.1.13 bicubic sampling, matching the RIS printer's numerical order.
//! Algorithm reference: SixLabors/ImageSharp v2.1.13 ResizeKernelMap/ResizeKernel.
//! Copyright Six Labors. Apache-2.0; see THIRD_PARTY_NOTICES.md.
use image::RgbaImage;
struct Kernel {
    start: usize,
    weights: Vec<f32>,
}
fn kernels(source: u32, target: u32) -> Vec<Kernel> {
    let ratio = source as f64 / target as f64;
    let scale = ratio.max(1.);
    let radius = (2. * scale).ceil();
    (0..target)
        .map(|i| {
            let center = (i as f64 + 0.5) * ratio - 0.5;
            let left = (center - radius).ceil().max(0.) as usize;
            let right = (center + radius).floor().min((source - 1) as f64) as usize;
            let weights: Vec<f64> = (left..=right)
                .map(|j| {
                    let x = (((j as f64) - center) / scale).abs() as f32;
                    (if x <= 1. {
                        ((1.5 * x - 2.5) * x * x) + 1.
                    } else if x < 2. {
                        (((-0.5 * x + 2.5) * x - 4.) * x) + 2.
                    } else {
                        0.
                    }) as f64
                })
                .collect();
            let sum: f64 = weights.iter().sum();
            Kernel {
                start: left,
                weights: weights
                    .into_iter()
                    .map(|v| (if sum > 0. { v / sum } else { v }) as f32)
                    .collect(),
            }
        })
        .collect()
}
fn convolve(k: &Kernel, pixel: impl Fn(usize) -> [f32; 4], fused: bool) -> [f32; 4] {
    let mut result = [0.; 4];
    if !fused {
        for (i, w) in k.weights.iter().enumerate() {
            let p = pixel(k.start + i);
            for j in 0..4 {
                result[j] += p[j] * w;
            }
        }
        return result;
    }
    // ImageSharp's AVX2 path uses two accumulators of two RGBA pixels each.
    let mut accum = [[0f32; 4]; 4];
    let length = k.weights.len();
    let mut i = 0;
    while i + 4 <= length {
        for (slot, a) in accum.iter_mut().enumerate() {
            let p = pixel(k.start + i + slot);
            for j in 0..4 {
                a[j] = p[j].mul_add(k.weights[i + slot], a[j]);
            }
        }
        i += 4;
    }
    let (low, high) = accum.split_at_mut(2);
    for (a, b) in low.iter_mut().zip(high) {
        for (a, b) in a.iter_mut().zip(b) {
            *a += *b;
        }
    }
    if i + 2 <= length {
        for (slot, accumulator) in accum.iter_mut().enumerate().take(2) {
            let p = pixel(k.start + i + slot);
            for j in 0..4 {
                accumulator[j] = p[j].mul_add(k.weights[i + slot], accumulator[j]);
            }
        }
        i += 2;
    }
    for j in 0..4 {
        result[j] = accum[0][j] + accum[1][j];
    }
    if i < length {
        let p = pixel(k.start + i);
        for j in 0..4 {
            result[j] = p[j].mul_add(k.weights[i], result[j]);
        }
    }
    result
}
pub(crate) fn resize(image: &RgbaImage, width: u32) -> RgbaImage {
    let height = (image.height() as f32 * (width as f32 / image.width() as f32))
        .round_ties_even()
        .max(1.) as u32;
    if width == image.width() && height == image.height() {
        return image.clone();
    }
    let horizontal = kernels(image.width(), width);
    let vertical = kernels(image.height(), height);
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let fused = std::is_x86_feature_detected!("avx2") && std::is_x86_feature_detected!("fma");
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    let fused = false;
    let mut first = vec![[0f32; 4]; (width * image.height()) as usize];
    for y in 0..image.height() {
        for x in 0..width {
            first[(y * width + x) as usize] = convolve(
                &horizontal[x as usize],
                |sx| {
                    let p = image.get_pixel(sx as u32, y);
                    let alpha = p[3] as f32 * (1. / 255.);
                    [
                        p[0] as f32 * (1. / 255.) * alpha,
                        p[1] as f32 * (1. / 255.) * alpha,
                        p[2] as f32 * (1. / 255.) * alpha,
                        alpha,
                    ]
                },
                fused,
            );
        }
    }
    RgbaImage::from_fn(width, height, |x, y| {
        let mut v = convolve(
            &vertical[y as usize],
            |sy| first[sy * width as usize + x as usize],
            fused,
        );
        let alpha = v[3].max(0.000001);
        for channel in &mut v[..3] {
            *channel /= alpha;
        }
        image::Rgba(v.map(|v| (v.clamp(0., 1.) * 255. + 0.5) as u8))
    })
}
