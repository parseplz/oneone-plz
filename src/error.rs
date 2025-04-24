use body_plz::reader::chunked_reader::ChunkReaderError;
use header_plz::error::HeaderReadError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpReadError {
    #[error("header read| {0}")]
    InfoLine(#[from] HeaderReadError),
    #[error("chunkreader| {0}")]
    ChunkReaderFailed(#[from] ChunkReaderError),
    // Not enough data
    #[error("chunk reader not enough data")]
    ChunkReaderNotEnoughData,
    #[error("header not enough data")]
    HeaderNotEnoughData,
}
