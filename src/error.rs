#[derive(Debug, Fail)]
pub enum ColferError {
    #[fail(display = "colfer: field {} exceeds {} bytes", field, overflow)]
    MaxSizeBreach {
        field: &'static str,
        overflow: usize,
    },
    #[fail(display = "colfer: field {} exceeds {} elements", field, overflow)]
    MaxListBreach {
        field: &'static str,
        overflow: usize,
    },
    #[fail(display = "colfer: unknown header at byte {}", byte)]
    UnknownHeader { byte: usize },
    #[fail(display = "colfer: data continuation at byte {}", byte)]
    Tail { byte: usize },
    #[fail(display = "colfer: unexpected empty buffer")]
    UnexpectedEof,
    #[fail(display = "colfer: unknown error")]
    Unknown,
}

pub type ColferResult<T> = Result<T, ColferError>;
