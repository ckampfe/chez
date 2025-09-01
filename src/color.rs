use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash)]
pub enum Color {
    #[serde(rename = "black")]
    Black,
    #[serde(rename = "white")]
    White,
}

impl Color {
    pub fn invert(&self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}
