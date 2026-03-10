use std::io::Write;

use exn::{Result, ResultExt};
use flate2::Compression;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to write the raw html in the encoder")]
    WriteAll,

    #[error(
        "Encoder failed to finish() aka failed to compress and write the compressed html in a buffer"
    )]
    Finish,
}

pub fn compress_html(html: &str) -> Result<Vec<u8>, Error> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(html.as_bytes())
        .or_raise(|| Error::WriteAll)?;
    let compressed_html = encoder.finish().or_raise(|| Error::Finish)?;

    Ok(compressed_html)
}
