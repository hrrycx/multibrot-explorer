use crate::hsl;
use crate::Color32;
use crate::ColoringMode;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
#[derive(Copy, Clone, PartialEq)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Complex {
    re: f64,
    im: f64,
}
fn cmul(z1: Complex, z2: Complex) -> Complex {
    Complex {
        re: z1.re * z2.re - z1.im * z2.im,
        im: z1.re * z2.im + z1.im * z2.re,
    }
}
fn cadd(z1: Complex, z2: Complex) -> Complex {
    Complex {
        re: z1.re + z2.re,
        im: z1.im + z2.im,
    }
}
fn crec(z: Complex) -> Complex {
    let den = z.re * z.re + z.im * z.im;
    return Complex {
        re: z.re / den,
        im: -z.im / den,
    };
}
fn cpow(z: Complex, n: i32) -> Complex {
    if n < 0 {
        return cpow(crec(z), -n);
    }
    if n == 0 {
        return Complex { re: 1., im: 0. };
    }
    if n == 1 {
        return z;
    }
    if n % 2 == 0 {
        return cpow(cmul(z, z), n / 2);
    } else {
        return cmul(z, cpow(cmul(z, z), (n - 1) / 2));
    }
}
pub fn mandelbrot(
    center: Coord,
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
fn hslcolor(iterations: i32, maxitr: f64, r: f64, shift: f64, range: f64) -> [u8; 4] {
    if iterations >= maxitr as i32 {
        return [0, 0, 0, 255];
    }
    let iterations: f64 = iterations as f64 + 1.0 - (((r).ln() / 2.0).ln()) / 2.0_f64.ln();
    let rgba = hsl::hsl_to_rgba(
        ((iterations / maxitr) + (shift / 360.) % 1.) * (range),
        1.,
        0.5,
    );
    rgba
}
fn monocolor(iterations: i32, maxitr: f64, r: f64, color: Color32, range: f64) -> [u8; 4] {
    if iterations >= maxitr as i32 {
        return [0, 0, 0, 255];
    }
    let iterations: f64 = iterations as f64 + 1.0 - (((r).ln() / 2.0).ln()) / 2.0_f64.ln();

    let mut color = color.to_array();
    for i in 0..3 {
        color[i] = ((iterations / maxitr) * color[i] as f64 * range) as u8;
    }
    color[3] = 255;
    color
}

fn funkycolor(iterations: i32, r: f64, shift: f64) -> [u8; 4] {
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
    return oy + (2.0 * ((y) / (height as f64 - 1.)) - 1.0) * 1.12 * scale;
}
fn mandel2(x0: f64, y0: f64, maxitr: f64) -> (i32, f64) {
    let mut iterations: i32 = 0;
    let mut x2: f64 = 0.0;
    let mut y2: f64 = 0.0;
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    let mut xold: f64 = 0.0;
    let mut yold: f64 = 0.0;

    let q = (x0 - 0.25).powi(2) + y0 * y0;
    if q * (q + (x0 - 0.25)) <= 0.25 * y0 * y0 {
        return (maxitr as i32, 0.0);
    }
    if (x0 + 1.).powi(2) + y0 * y0 <= 0.0625 {
        return (maxitr as i32, 0.0);
    }
    while x2 + y2 < 4.0 && iterations < maxitr as i32 {
        y = y * (x + x) + y0;
        x = x2 - y2 + x0;
        x2 = x * x;
        y2 = y * y;
        if x == xold && y == yold {
            return (maxitr as i32, x2 + y2);
        }
        if iterations % 25 == 0 {
            xold = x;
            yold = y;
        }
        iterations = iterations + 1;
    }
    return (iterations, x2 + y2);
}
fn renderline(
    linenumber: u32,
    scale: f64,
    maxitr: f64,
    center: Coord,
    exponent: i32,
    width: i32,
    height: i32,
    mode: ColoringMode,
) -> Vec<u8> {
    let mut x0: f64;
    let mut line: Vec<u8> = Vec::new();
    let y0 = py(linenumber as f64, scale, center.y, height);
    for x in 0..width {
        x0 = px(x as f64, scale, center.x, width);
        let (iterations, r) = mandelcomp(x0, y0, maxitr, exponent);
        match mode {
            ColoringMode::Hsl(shift, range) => {
                line.extend_from_slice(&hslcolor(iterations, maxitr, r, shift, range))
            }
            ColoringMode::Monochrome(color, range) => {
                line.extend_from_slice(&monocolor(iterations, maxitr, r, color, range))
            }
            ColoringMode::Funky(shift) => line.extend_from_slice(&funkycolor(iterations, r, shift)),
        }
    }
    return line;
}
pub fn mandelcomp(x0: f64, y0: f64, maxitr: f64, n: i32) -> (i32, f64) {
    if n == 2 {
        return mandel2(x0, y0, maxitr);
    }
    let mut zold: Complex = Complex { re: 0., im: 0. };
    let mut iterations: i32 = 0;
    let c = Complex { re: x0, im: y0 };
    let mut z: Complex = Complex { re: 0., im: 0. };
    while z.re * z.re + z.im * z.im < 4. && iterations < maxitr as i32 {
        z = cadd(cpow(z, n), c);
        if z == zold {
            return (maxitr as i32, 0.);
        }
        if iterations % 25 == 0 {
            zold = z;
        }
        iterations += 1;
    }
    return (iterations, z.re * z.re + z.im * z.im);
}
pub fn mandelcomplist(x0: f64, y0: f64, maxitr: f64, n: i32) -> (i32, Vec<Coord>, i32) {
    let mut points: Vec<Coord> = Vec::new();
    let mut iterations: i32 = 0;
    let c = Complex { re: x0, im: y0 };
    let mut z: Complex = Complex { re: 0., im: 0. };
    while z.re * z.re + z.im * z.im < 1000. && iterations < maxitr as i32 {
        z = cadd(cpow(z, n), c);
        iterations += 1;
        if points.contains(&Coord { x: z.re, y: z.im }) {
            return (maxitr as i32, points, iterations);
        } else {
            points.push(Coord { x: z.re, y: z.im });
        }
    }
    return (iterations, points, -1);
}
pub fn piapprox() -> f64 {
    let epsilon = 0.0000001;
    let (iter, _) = mandelcomp(-0.75, epsilon, 1000000000000000000000000000000000., 2);
    return iter as f64 * epsilon;
}
pub fn xp(x: f64, ox: f64, zoom: f64, width: i32) -> i32 {
    return ((((((x - ox) / zoom) / 1.235) + 1.) / 2.) * width as f64) as i32;
}
pub fn yp(y: f64, oy: f64, zoom: f64, height: i32) -> i32 {
    return ((((((y - oy) / zoom) / 1.12) + 1.) / 2.) * height as f64) as i32;
}
