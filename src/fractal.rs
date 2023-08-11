use crate::Color32;
use crate::ColoringMode;
use colorsys::Hsl;
use colorsys::Rgb;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
#[derive(Copy, Clone, PartialEq)]
pub struct coord {
    pub x: f64,
    pub y: f64,
}
#[derive(Copy, Clone, Debug)]
pub struct complex {
    re: f64,
    im: f64,
}
fn cmul(z1: complex, z2: complex) -> complex {
    complex {
        re: z1.re * z2.re - z1.im * z2.im,
        im: z1.re * z2.im + z1.im * z2.re,
    }
}
fn cadd(z1: complex, z2: complex) -> complex {
    complex {
        re: z1.re + z2.re,
        im: z1.im + z2.im,
    }
}
fn crec(z: complex) -> complex {
    let den = z.re * z.re + z.im * z.im;
    return complex {
        re: z.re / den,
        im: -z.im / den,
    };
}
fn cpow(z: complex, n: i32) -> complex {
    if (n < 0) {
        return cpow(crec(z), -n);
    }
    if (n == 0) {
        return complex { re: 1., im: 0. };
    }
    if (n == 1) {
        return z;
    }
    if (n % 2 == 0) {
        return cpow(cmul(z, z), n / 2);
    } else {
        return cmul(z, cpow(cmul(z, z), (n - 1) / 2));
    }
}
pub fn mandelbrot(
    center: coord,
    scale: f64,
    maxitr: f64,
    exponent: i32,
    width: i32,
    height: i32,
    mode: ColoringMode,
) -> Vec<u8> {
    let buf: Vec<u8> = (0..height)
        .into_par_iter()
        .map(|y| {
            renderline(
                y as u32, scale, maxitr, center, exponent, width, height, mode,
            )
        })
        .flatten()
        .collect();
    buf
}
fn hslcolor(iterations: i32, maxitr: f64, r: f64, shift: f64) -> [u8; 4] {
    if iterations >= maxitr as i32 {
        return [0, 0, 0, 255];
    }
    let iterations: f64 = iterations as f64 + 1.0 - (((r).ln() / 2.0).ln()) / 2.0_f64.ln();
    let mut hsl = Hsl::default();
    hsl.set_hue((360.0 * (iterations / maxitr) + shift) % 360.0);
    hsl.set_saturation(100.);
    hsl.set_lightness(50.0);
    let rgb_arr: [u8; 3] = Rgb::from(&hsl).into();
    let mut rgba: [u8; 4] = [0; 4];
    rgba[0] = rgb_arr[0];
    rgba[1] = rgb_arr[1];
    rgba[2] = rgb_arr[2];
    rgba[3] = 255;
    rgba
}
fn monocolor(iterations: i32, maxitr: f64, r: f64, color: Color32) -> [u8; 4] {
    if iterations >= maxitr as i32 {
        return [0, 0, 0, 255];
    }
    let iterations: f64 = iterations as f64 + 1.0 - (((r).ln() / 2.0).ln()) / 2.0_f64.ln();

    let mut color = color.to_array();
    for i in 0..3 {
        color[i] = ((iterations / maxitr) * color[i] as f64) as u8;
    }
    color[3] = 255;
    color
}
pub fn px(x: f64, scale: f64, oX: f64, width: i32) -> f64 {
    return (oX) + ((2.0 * ((x - 1.0) / (width as f64 - 1.0)) - 1.0) * 1.235 * scale);
}
pub fn py(y: f64, scale: f64, oY: f64, height: i32) -> f64 {
    return (oY + (2.0 * ((y - 1.0) / (height as f64 - 1.0)) - 1.0) * 1.12 * scale);
}
fn mandel2(mut x0: f64, mut y0: f64, maxitr: f64) -> (i32, f64) {
    let mut iterations: i32 = 0;
    let mut x2: f64 = 0.0;
    let mut y2: f64 = 0.0;
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    while x2 + y2 < 4.0 && iterations < maxitr as i32 {
        y = 2.0 * x * y + y0;
        x = x2 - y2 + x0;
        x2 = x * x;
        y2 = y * y;
        iterations = iterations + 1;
    }
    return (iterations, x2 + y2);
}
fn renderline(
    linenumber: u32,
    scale: f64,
    maxitr: f64,
    center: coord,
    exponent: i32,
    width: i32,
    height: i32,
    mode: ColoringMode,
) -> (Vec<u8>) {
    let mut x0: f64;
    let mut line: Vec<u8> = Vec::new();
    let y0 = py((linenumber as f64), scale, center.y, height);
    for x in 0..width {
        x0 = px(x as f64, scale, center.x, width);
        let (iterations, r) = mandelcomp(x0, y0, maxitr, exponent);
        match mode {
            ColoringMode::Hsl(shift) => {
                line.extend_from_slice(&hslcolor(iterations, maxitr, r, shift))
            }
            ColoringMode::Monochrome(color) => {
                line.extend_from_slice(&monocolor(iterations, maxitr, r, color))
            }
        }
    }
    return line;
}
pub fn mandelcomp(mut x0: f64, mut y0: f64, maxitr: f64, n: i32) -> (i32, f64) {
    if n == 2 {
        return mandel2(x0, y0, maxitr);
    }
    let mut iterations: i32 = 0;
    let c = complex { re: x0, im: y0 };
    let mut z: complex = complex { re: 0., im: 0. };
    while z.re * z.re + z.im * z.im < 4. && iterations < maxitr as i32 {
        z = cadd(cpow(z, n), c);
        iterations += 1;
    }
    return (iterations, z.re * z.re + z.im * z.im);
}
pub fn piapprox() -> f64{
    let epsilon = 0.0000001;
    let (iter,_) = mandelcomp(-0.75, epsilon, 1000000000000000000000000000000000., 2);
    return iter as f64 * epsilon;
}
