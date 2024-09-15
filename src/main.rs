use std::str::FromStr;
use std::fs::File;
use std::env;
use num::Complex;
use image::ImageEncoder;
use image::error::ImageResult;
use image::codecs::png::PngEncoder;


const LIMIT_TO_CALL_IT_OFF_TO_INFINITY: f64 = 4.0;
const LIMIT_OF_ITERATION: usize = 255;
const CMD_ARG_COMPLEX_NUMBER_SEPARATOR: char = ',';


fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize>
{
    let mut z = Complex { re: 0.0, im: 0.0 };

    for i in 0..limit {
        if z.norm_sqr() > LIMIT_TO_CALL_IT_OFF_TO_INFINITY {
            return Some(i);
        }
        z = z * z + c;
    }

    None
}

fn parse_bool(s: &str) -> Option<bool> {
    match s {
        "0" => Some(false),
        "1" => Some(true),
        _ => None,
    }
}

fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)>
{
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None,
            }
        }
    }
}

fn parse_complex(s: &str) -> Option<Complex<f64>>
{
    match parse_pair(s, CMD_ARG_COMPLEX_NUMBER_SEPARATOR) {
        None => None,
        Some((re, im)) => Some(Complex {re, im}),
    }
}

fn pixel_to_point(bounds: (usize, usize), pixel: (usize, usize),
                    upper_left: Complex<f64>, lower_right: Complex<f64>) -> Complex<f64>
{
    let (width, height) = (lower_right.re - upper_left.re, upper_left.im - lower_right.im);
    Complex {
        re: upper_left.re + pixel.0 as f64 * (width  / bounds.0 as f64),
        im: upper_left.im - pixel.1 as f64 * (height / bounds.1 as f64),
    }
}

fn render(pixels: &mut [u8], bounds: (usize, usize), upper_left: Complex<f64>, lower_right: Complex<f64>)
{
    assert!(pixels.len() == bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let pixel = (column, row);
            let point = pixel_to_point(bounds, pixel, upper_left, lower_right);
            pixels[(row * bounds.0) + column] = 
                match  escape_time(point, LIMIT_OF_ITERATION) {
                    None => 0,
                    Some(count) => (LIMIT_OF_ITERATION - count) as u8,
                };
        }
    }
}

fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize)) -> ImageResult<()>
{
    let output = File::create(filename)?;

    let encoder = PngEncoder::new(output);
    let (width, height) = (bounds.0 as u32, bounds.1 as u32);
    encoder.write_image(&pixels, width, height, image::ExtendedColorType::L8)?;
    Ok(())
}

fn run_sequentially(filename: &str, bounds: (usize, usize), upper_left: Complex<f64>, lower_right: Complex<f64>) {
    let (width, height) = bounds;
    let mut pixels = vec![0; width * height];
    render(&mut pixels, bounds, upper_left, lower_right);
    write_image(filename, &pixels, bounds).expect("error writing PNG file");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 6 {
        eprintln!("Usage: {} FILE PIXELS UPPERLEFT LOWERRIGHT <SEQUENTIAL:0|PARALLEL:1>", args[0]);
        eprintln!("Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20 1", args[0]);
        std::process::exit(1);
    }

    let bounds: (usize, usize) = parse_pair::<usize>(&args[2], 'x').expect("error parsing image dimensions");
    let upper_left: Complex<f64> = parse_complex(&args[3]).expect("error parsing upper left corner point");
    let lower_right: Complex<f64> = parse_complex(&args[4]).expect("error parsing lower right corner point");
    let heuristics: bool = parse_bool(&args[5]).expect("error parsing <SEQUENTIAL:0|PARALLEL:1>");

    if heuristics {
        run_sequentially(&args[1], bounds, upper_left, lower_right);
    } else {
        run_sequentially(&args[1], bounds, upper_left, lower_right);
    }
}
