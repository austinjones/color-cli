use std::path::{Path, PathBuf};
use rand::{thread_rng, Rng};
use std::{env, io};
use std::io::{BufWriter, BufReader};
use std::fs::File;
use clap::Clap;
use rand::seq::SliceRandom;

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "Austin Jones")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}


#[derive(Clap)]
enum SubCommand {
    /// A help message for the Test subcommand
    #[clap(name = "extract", version = "0.1", author = "Austin Jones")]
    Extract(Extract),

    #[clap(name = "render", version = "0.1", author = "Austin Jones")]
    Render(Render)
}


/// A subcommand for extracting colors from an image
#[derive(Clap)]
struct Extract {
    #[clap(name = "input", index=1)]
    input: PathBuf,

    #[clap(short = "p", default_value="20000")]
    n_samples: usize
}

/// A subcommand for rendering colors from CSV into an image
#[derive(Clap)]
struct Render {
    #[clap(name = "input", index=1)]
    input: PathBuf,

    /// Print debug info
    #[clap(short = "p", default_value="800")]
    pixels: usize,

    #[clap(short = "n", default_value="20")]
    cells: usize
}


fn main() -> Result<(), io::Error>{
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Extract(args) => extract(args)?,
        SubCommand::Render(args) => render(args)?
    };

    Ok(())
}

fn render(args: Render) -> Result<(), io::Error> {
    let input = File::open(&args.input)?;
    let output = output_file(&args.input, "jpg");

    let mut reader = csv::Reader::from_reader(BufReader::new(input));
    let mut all_colors = Vec::new();

    println!("Reading colors from {}", args.input.to_str().unwrap());

    for record in reader.records() {
        let record = record?;
        let r = record.get(0).unwrap();
        let g = record.get(1).unwrap();
        let b = record.get(2).unwrap();

        let rf: f64 = r.parse().unwrap();
        let gf: f64 = g.parse().unwrap();
        let bf: f64 = b.parse().unwrap();

        let ru8 = (rf * 255.0) as u8;
        let gu8 = (gf * 255.0) as u8;
        let bu8 = (bf * 255.0) as u8;

        all_colors.push(image::Rgb([ru8, gu8, bu8]));
    }

    let cells = args.cells;

    let pixels = args.pixels;
    let cell_width = pixels as f64 / cells as f64;

    let mut image = image::RgbImage::new(pixels as u32, pixels as u32);
    for y in 0..cells {
        for x in 0..cells {
            let color = all_colors.choose(&mut thread_rng()).unwrap();
            let y_min = (y as f64 * cell_width).floor() as u32;
            let y_max = ((y+1) as f64 * cell_width).floor() as u32;

            let x_min = (x as f64 * cell_width).floor() as u32;
            let x_max = ((x+1) as f64 * cell_width).floor() as u32;

            for image_y in y_min..y_max {
                for image_x in x_min..x_max {
                    image.put_pixel(image_x, image_y, color.clone());
                }
            }
        }
    }

    println!("Writing output to  {}", output.to_str().unwrap());

    image.save(&output).unwrap();

    Ok(())
}

fn extract(args: Extract) -> Result<(), io::Error> {
    println!("Collecting colors from {}", args.input.to_str().unwrap());

    let image = image::open(&args.input).unwrap();
    let image = image.into_rgba();

    let output = output_file(&args.input, "csv");
    let output_file = File::create(&output)?;
    let mut output_writer = csv::Writer::from_writer(BufWriter::new(output_file));

    println!("Writing output to  {}", output.to_str().unwrap());

    let (max_w, max_h) = image.dimensions();

    let mut rng = thread_rng();

    for _ in 0..args.n_samples {
        let x = rng.gen_range(0, max_w);
        let y =  rng.gen_range(0, max_h);

        let pix = image.get_pixel(x, y);

        let r = ((pix.0[0] as f32) + rng.gen_range(-0.5, 0.5)) / 255.0;
        let g = ((pix.0[1] as f32) + rng.gen_range(-0.5, 0.5)) / 255.0;
        let b = ((pix.0[2] as f32) + rng.gen_range(-0.5, 0.5)) / 255.0;

        output_writer.write_record(&[r.to_string(), g.to_string(), b.to_string()])?;
    }

    println!("Output written to {}", output.to_str().unwrap());

    Ok(())
}

fn output_file(input: &Path, ext: &str) -> PathBuf {
    let dir = env::current_dir().unwrap();
    let mut output = output_path(&dir, input, ext, None);

    if output.exists() {
        let mut counter = 1;

        while output.exists() {
            output = output_path(&dir, input, ext, Some(counter));
            counter += 1;
        }
    }

    output
}

fn output_path(dir: &Path, input_file: &Path, ext: &str, counter: Option<usize>) -> PathBuf {
    let mut path = dir.to_path_buf();

    let filestem = input_file
        .file_stem().unwrap()
        .to_str().unwrap()
        .to_string();
    let output_filename = match counter {
        Some(c) => filestem + "-" + &c.to_string() + "." + ext,
        None => filestem + "." + ext
    };

    path.push(output_filename);

    path
}