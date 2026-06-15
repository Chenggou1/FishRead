use ulid::Ulid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BookId(pub String);

impl BookId {
    pub fn new() -> Self {
        Self(format!("book_{}", Ulid::new()))
    }
}

impl Default for BookId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp(pub i64);

impl Timestamp {
    pub fn now() -> Self {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self(secs as i64)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookFormat {
    Epub,
}

impl BookFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Epub => "epub",
        }
    }
}

impl TryFrom<&str> for BookFormat {
    type Error = crate::error::FishReadError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "epub" => Ok(Self::Epub),
            other => Err(crate::error::FishReadError::UnsupportedFormat(
                other.to_owned(),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Book {
    pub id: BookId,
    pub title: String,
    pub author: Option<String>,
    pub format: BookFormat,
    pub source_path: Option<String>,
    pub imported_at: Timestamp,
    pub updated_at: Timestamp,
}
