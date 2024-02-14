use std::io::{Cursor, Write};

use anyhow::Context as _;
use clap::Parser;
use clio::{Input, Output};
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

    /// The path or URL of the input image (or '-' for STDIN).
    input: Input,

    /// The path to write the shitpic to, the URL to post the shitpic to, or '-' for STDOUT.
    #[arg(default_value = "-")]
    output: Output,
}

fn main() -> anyhow::Result<()> {
    let mut opts = Opts::parse();
    let input_img = read_image(&mut opts.input)
        .with_context(|| format!("reading image from {} failed", opts.input))?;

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

    write_output(&mut opts.output, &encode_buf)
        .with_context(|| format!("writing image to {} failed", opts.output))?;

    Ok(())
}

fn read_image(input: &mut Input) -> anyhow::Result<DynamicImage> {
    // Buffer image into memory so we can taste what the format is.
    let mut data = Vec::with_capacity(1024 * 1024 * 10);
    input
        .lock()
        .read_to_end(&mut data)
        .context("reading input data")?;

    let reader = ImageReader::new(Cursor::new(data));
    let reader = reader
        .with_guessed_format()
        .context("guessing input format")?;

    reader.decode().context("decoding image")
}

fn write_output(output: &mut Output, data: &[u8]) -> anyhow::Result<()> {
    output
        .lock()
        .write_all(data)
        .context("writing output")
}
