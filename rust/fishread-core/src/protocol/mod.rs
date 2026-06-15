pub mod dto;

use serde::Serialize;

pub use dto::{BookDto, BookListDto, BookUseDto, ImportResultDto, ImportWarningDto, PositionDto};

use crate::error::FishReadError;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorBody>,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
        }
    }
}

impl ApiResponse<()> {
    pub fn err(err: &FishReadError) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(ErrorBody {
                code: err.code().to_owned(),
                message: err.to_string(),
            }),
        }
    }

    pub fn internal_err(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(ErrorBody {
                code: "INTERNAL_ERROR".to_owned(),
                message: msg.into(),
            }),
        }
    }
}

/// DTO for `fishread init`
#[derive(Debug, Serialize)]
pub struct InitData {
    pub initialized: bool,
    pub database_path: String,
}
