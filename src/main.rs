use std::{
    io::{stdout, Write},
    path::{Path, PathBuf},
};

use anyhow::Context as _;
use clap::Parser;
use image::{
    codecs::jpeg::{JpegDecoder, JpegEncoder},
    io::Reader as ImageReader,
    DynamicImage, ImageDecoder, ImageEncoder,
};

static SHITPIC_PANIC: &str = "shitpic compression produced an invalid jpeg. this is a bug!";

/// Mr. Evrart is helping me with my memes.
#[derive(Debug, Parser)]
struct Opts {
    /// The number of rounds of jpeg compression to apply.
    #[arg(
        long,
        default_value_t = 69,
        value_parser = clap::value_parser!(u64).range(1..),
    )]
    rounds: u64,

    /// The quality of JPEG to apply. Choose a lower value
    /// for shittier pics, faster.
    #[arg(
        long,
        default_value_t = 10,
        value_parser = clap::value_parser!(u8).range(1..=50),
    )]
    quality: u8,

    /// The path to the input image.
    input_path: PathBuf,

    /// The path to write the shitpic to.
    output_path: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    let input_img = read_image(&opts.input_path)?;

    let buf_len = input_img.as_bytes().len();
    let mut decode_buf = Vec::with_capacity(buf_len);
    let mut encode_buf = Vec::with_capacity(buf_len);
    let mut img_buf = Vec::with_capacity(buf_len);

    // step one: convert anything to jpeg.
    //
    // encode to the encode buf and then swap the bufs around so that the
    // newly encoded data is ready in decode_buf
    {
        let enc = JpegEncoder::new_with_quality(&mut encode_buf, opts.quality);
        input_img
            .write_with_encoder(enc)
            .context("encoding jpeg failed")?;
    }

    // step two: start the volumetric shit compressor
    for _ in 0..opts.rounds {
        std::mem::swap(&mut encode_buf, &mut decode_buf);
        let dc = JpegDecoder::new(&decode_buf[..]).expect(SHITPIC_PANIC);

        let jpeg_bytes = dc.total_bytes() as usize;
        img_buf.resize(jpeg_bytes, 0);

        let color_type = dc.color_type();
        let (width, height) = dc.dimensions();
        let img = &mut img_buf[0..jpeg_bytes];

        dc.read_image(img).expect(SHITPIC_PANIC);

        let ec = JpegEncoder::new_with_quality(&mut encode_buf, opts.quality);
        ec.write_image(img, width, height, color_type).unwrap();
    }

    write_output(opts.output_path, &encode_buf)?;

    Ok(())
}

fn read_image<P: AsRef<Path>>(input_path: P) -> anyhow::Result<DynamicImage> {
    let input_path = input_path.as_ref();
    let reader = ImageReader::open(input_path)
        .with_context(|| format!("failed to open input file: {}", input_path.display()))?;

    let reader = reader
        .with_guessed_format()
        .context("error guessing input format")?;

    reader.decode().context("decoding image failed")
}

fn write_output<P: AsRef<Path>>(output_path: Option<P>, data: &[u8]) -> anyhow::Result<()> {
    let output_path = output_path.as_ref().map(|p| p.as_ref());
    // match and then dispatch, intead of returning some kind of boxed writer
    // and doing dynamic dispatch that way.
    match output_path {
        None => stdout().write_all(data).context("writing image failed"),
        Some(path) => std::fs::write(path, data)
            .with_context(|| format!("writing image to {} failed", path.display())),
    }
}
