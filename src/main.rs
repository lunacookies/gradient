use image::png::PngEncoder;
use image::{ColorType, EncodableLayout, Rgb, RgbImage};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::env;
use std::io::{self, Write};
use std::ops::Range;
use tincture::{ColorSpace, Hue, LinearRgb, Oklab, Oklch, Srgb};

fn main() -> anyhow::Result<()> {
    let mut args = env::args().skip(1);

    let top = Oklch {
        l: args.next().unwrap().parse().unwrap(),
        c: args.next().unwrap().parse().unwrap(),
        h: Hue::from_degrees(args.next().unwrap().parse().unwrap()).unwrap(),
    }
    .into();

    let bottom = Oklch {
        l: args.next().unwrap().parse().unwrap(),
        c: args.next().unwrap().parse().unwrap(),
        h: Hue::from_degrees(args.next().unwrap().parse().unwrap()).unwrap(),
    }
    .into();

    let width = args.next().unwrap().parse().unwrap();
    let height = args.next().unwrap().parse().unwrap();
    let graininess = args.next().unwrap().parse().unwrap();

    let mut rng = rand::thread_rng();

    let image = RgbImage::from_fn(width, height, |_, y| {
        let color = gen_color(y, height, &mut rng, bottom, top, graininess);
        convert(color)
    });

    let mut stdout = io::BufWriter::new(io::stdout());
    let encoder = PngEncoder::new(&mut stdout);
    encoder.encode(image.as_bytes(), width, height, ColorType::Rgb8)?;

    stdout.flush()?;

    Ok(())
}

fn gen_color(
    y: u32,
    height: u32,
    rng: &mut ThreadRng,
    bottom: Oklab,
    top: Oklab,
    graininess: f32,
) -> Oklab {
    let progress = y as f32 / height as f32;
    let binary = if rng.gen_bool(progress as f64) { bottom } else { top };
    let smooth = blend_oklab(top, bottom, progress);

    blend_oklab(smooth, binary, graininess)
}

fn convert(oklab: Oklab) -> Rgb<u8> {
    let linear_rgb: LinearRgb = tincture::convert(oklab);
    let srgb = Srgb::from(linear_rgb);
    assert!(srgb.in_bounds());

    let r = (srgb.r * 255.0) as u8;
    let g = (srgb.g * 255.0) as u8;
    let b = (srgb.b * 255.0) as u8;

    Rgb([r, g, b])
}

fn blend_oklab(first: Oklab, second: Oklab, x: f32) -> Oklab {
    Oklab {
        l: lerp(x, first.l..second.l),
        a: lerp(x, first.a..second.a),
        b: lerp(x, first.b..second.b),
    }
}

fn lerp(x: f32, range: Range<f32>) -> f32 {
    x * (range.end - range.start) + range.start
}
