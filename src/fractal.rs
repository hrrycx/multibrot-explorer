use crate::hsl;
use crate::Color32;
use crate::ColoringMode;
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
    let rgba = hsl::hsl_to_rgba((iterations / maxitr) + (shift/360.)  % 1., 1., 0.5);
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

fn funkycolor(iterations: i32, maxitr: f64, r: f64, shift: f64) -> [u8; 4]{
    if r <= 4. {
        return [0, 0, 0, 255];
    }
    let rgba = hsl::hsl_to_rgba(shift / 360. + (iterations as f64 / 800. * r), 1., 0.5);
    rgba
}

pub fn px(x: f64, scale: f64, ox: f64, width: i32) -> f64 {
    return (ox) + ((2.0 * ((x) / (width as f64 - 1.)) - 1.0) * 1.235 * scale);
}
pub fn py(y: f64, scale: f64, oy: f64, height: i32) -> f64 {
    return (oy + (2.0 * ((y) / (height as f64 - 1.)) - 1.0) * 1.12 * scale);
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
            ColoringMode::Funky(shift) =>{
                line.extend_from_slice(&funkycolor(iterations, maxitr, r, shift))
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
pub fn xp(x: f64, ox: f64, zoom: f64, width: i32)-> i32
{
    return ((((((x-ox) / zoom) / 1.235) + 1.) / 2.) * width as f64) as i32
}
pub fn yp(y: f64, oy: f64, zoom: f64, height: i32)-> i32
{
    return ((((((y-oy) / zoom) / 1.12) + 1.) / 2.) * height as f64) as i32
}
