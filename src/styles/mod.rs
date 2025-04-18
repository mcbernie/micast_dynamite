mod taffy;
use std::collections::HashMap;
use std::str::FromStr;
use std::num::ParseFloatError;

use ::taffy::{Display, LengthPercentage};

use crate::parse_color;
use crate::parser::parse_styles;

/// Die zentrale Struktur, die die für dein Layout relevanten Style-Eigenschaften kapselt.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Style {
    pub margin: Option<EdgeValues>,
    pub padding: Option<EdgeValues>,
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub display: Option<Display>,
    pub flex_direction: Option<FlexDirection>,
    pub font_size: Option<f32>,
    pub background_color: Option<[u8; 4]>,
    pub color: Option<[u8; 4]>,
    pub gap: Option<Dimension>,
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<AlignContent>,
    // Weitere Eigenschaften können hier ergänzt werden.
}

/// Repräsentiert Werte für margin oder padding. Unterstützt CSS-Shorthand (1 bis 4 Werte).
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeValues {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeValues {
    /// Parst einen CSS-Wert, z. B. "10" oder "10 20" oder "10 20 30 40".
    /// Hier wird nur ein Basis-Parsing durchgeführt (Einheit ignoriert, angenommen wird px).
    pub fn from_str(s: &str) -> Result<Self, ParseFloatError> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        match parts.len() {
            1 => {
                let v = parse_length(parts[0])?;
                Ok(Self { top: v, right: v, bottom: v, left: v })
            }
            2 => {
                let v1 = parse_length(parts[0])?;
                let v2 = parse_length(parts[1])?;
                Ok(Self { top: v1, right: v2, bottom: v1, left: v2 })
            }
            3 => {
                let top = parse_length(parts[0])?;
                let right_left = parse_length(parts[1])?;
                let bottom = parse_length(parts[2])?;
                Ok(Self { top, right: right_left, bottom, left: right_left })
            }
            4 => {
                let top = parse_length(parts[0])?;
                let right = parse_length(parts[1])?;
                let bottom = parse_length(parts[2])?;
                let left = parse_length(parts[3])?;
                Ok(Self { top, right, bottom, left })
            }
            _ => {
                // Bei ungültigen Angaben kann man auch ein Default (z. B. 0) zurückgeben oder einen Fehler liefern.
                Ok(Self { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 })
            }
        }
    }
}

/// Hilfsfunktion, die einen Längenwert (aktuell ohne explizite Einheit) parst.
/// In einer erweiterten Version kannst du hier auch Einheiten wie %, em, etc. unterstützen.
fn parse_length(s: &str) -> Result<f32, ParseFloatError> {
    // Extrahiere nur den Zahlenanteil, ignoriert etwaige Einheiten
    let number_part: String = s.chars().take_while(|c| c.is_digit(10) || *c == '.').collect();

    number_part.trim_end_matches("pt").trim_end_matches("px").trim().parse::<f32>()
}

/// Repräsentiert Dimensionen, die entweder als feste Punkte (px), Prozentwerte oder "auto" angegeben werden.
#[derive(Debug, Clone, PartialEq)]
pub enum Dimension {
    Auto,
    Points(f32),
    Percent(f32),
}

impl FromStr for Dimension {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.eq_ignore_ascii_case("auto") {
            Ok(Dimension::Auto)
        } else if s.ends_with('%') {
            let num = s.trim_end_matches('%').trim().parse::<f32>()?;
            Ok(Dimension::Percent(num))
        } else if s.ends_with("px") {
            let num = s.trim_end_matches("px").trim().parse::<f32>()?;
            Ok(Dimension::Points(num))
        } else if s.ends_with("pt") {
            let num = s.trim_end_matches("pt").trim().parse::<f32>()?;
            Ok(Dimension::Points(num))
        } else {
            let num = s.trim().parse::<f32>()?;
            Ok(Dimension::Points(num))
        }
    }
}

impl Into<LengthPercentage> for Dimension {
    fn into(self) -> LengthPercentage {
        match self {
            Dimension::Auto => LengthPercentage::Length(0.0),
            Dimension::Points(val) => LengthPercentage::Length(val),
            Dimension::Percent(val) => LengthPercentage::Percent(val / 100.0),
        }
    }
}

/// Enum zur Repräsentation der Flex-Richtung.
#[derive(Debug, Clone, PartialEq)]
pub enum FlexDirection {
    Row,
    Column,
    // Weitere Richtungen (z.B. RowReverse, ColumnReverse) können ergänzt werden.
}

impl FromStr for FlexDirection {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "row" => Ok(FlexDirection::Row),
            "column" => Ok(FlexDirection::Column),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlignItems {
    /// Items are packed toward the start of the axis
    Start,
    /// Items are packed toward the end of the axis
    End,
    /// Items are packed towards the flex-relative start of the axis.
    ///
    /// For flex containers with flex_direction RowReverse or ColumnReverse this is equivalent
    /// to End. In all other cases it is equivalent to Start.
    FlexStart,
    /// Items are packed towards the flex-relative end of the axis.
    ///
    /// For flex containers with flex_direction RowReverse or ColumnReverse this is equivalent
    /// to Start. In all other cases it is equivalent to End.
    FlexEnd,
    /// Items are packed along the center of the cross axis
    Center,
    /// Items are aligned such as their baselines align
    Baseline,
    /// Stretch to fill the container
    Stretch,
}

impl FromStr for AlignItems {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "start" => Ok(AlignItems::Start),
            "end" => Ok(AlignItems::End),
            "flex-start" => Ok(AlignItems::FlexStart),
            "flex-end" => Ok(AlignItems::FlexEnd),
            "center" => Ok(AlignItems::Center),
            "baseline" => Ok(AlignItems::Baseline),
            "stretch" => Ok(AlignItems::Stretch),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlignContent {
    /// Items are packed toward the start of the axis
    Start,
    /// Items are packed toward the end of the axis
    End,
    /// Items are packed towards the flex-relative start of the axis.
    ///
    /// For flex containers with flex_direction RowReverse or ColumnReverse this is equivalent
    /// to End. In all other cases it is equivalent to Start.
    FlexStart,
    /// Items are packed towards the flex-relative end of the axis.
    ///
    /// For flex containers with flex_direction RowReverse or ColumnReverse this is equivalent
    /// to Start. In all other cases it is equivalent to End.
    FlexEnd,
    /// Items are centered around the middle of the axis
    Center,
    /// Items are stretched to fill the container
    Stretch,
    /// The first and last items are aligned flush with the edges of the container (no gap)
    /// The gap between items is distributed evenly.
    SpaceBetween,
    /// The gap between the first and last items is exactly THE SAME as the gap between items.
    /// The gaps are distributed evenly
    SpaceEvenly,
    /// The gap between the first and last items is exactly HALF the gap between items.
    /// The gaps are distributed evenly in proportion to these ratios.
    SpaceAround,
}

impl FromStr for AlignContent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "start" => Ok(AlignContent::Start),
            "end" => Ok(AlignContent::End),
            "flex-start" => Ok(AlignContent::FlexStart),
            "flex-end" => Ok(AlignContent::FlexEnd),
            "center" => Ok(AlignContent::Center),
            "stretch" => Ok(AlignContent::Stretch),
            "space-between" => Ok(AlignContent::SpaceBetween),
            "space-evenly" => Ok(AlignContent::SpaceEvenly),
            "space-around" => Ok(AlignContent::SpaceAround),
            _ => Err(()),
        }
    }
}


/// Hilfsfunktion, um EdgeValues aus einer HashMap zu parsen.
/// Zuerst wird der Shorthand-Wert (z. B. "margin") genutzt, dann werden
/// individuelle Werte ("margin-top", etc.) übersteuert.
fn parse_edge_from_hashmap(map: &HashMap<String, String>, property: &str) -> Option<EdgeValues> {
    let shorthand = map.get(property).and_then(|s| EdgeValues::from_str(s).ok());
    let mut top = shorthand.as_ref().map(|e| e.top).unwrap_or(0.0);
    let mut right = shorthand.as_ref().map(|e| e.right).unwrap_or(0.0);
    let mut bottom = shorthand.as_ref().map(|e| e.bottom).unwrap_or(0.0);
    let mut left = shorthand.as_ref().map(|e| e.left).unwrap_or(0.0);

    let mut found = shorthand.is_some();

    if let Some(s) = map.get(&format!("{}-top", property)) {
        if let Ok(v) = parse_length(s) {
            top = v;
            found = true;
        }
    }
    if let Some(s) = map.get(&format!("{}-right", property)) {
        if let Ok(v) = parse_length(s) {
            right = v;
            found = true;
        }
    }
    if let Some(s) = map.get(&format!("{}-bottom", property)) {
        if let Ok(v) = parse_length(s) {
            bottom = v;
            found = true;
        }
    }
    if let Some(s) = map.get(&format!("{}-left", property)) {
        if let Ok(v) = parse_length(s) {
            left = v;
            found = true;
        }
    }

    if found {
        Some(EdgeValues { top, right, bottom, left })
    } else {
        None
    }
}


impl Style {
    /// Erstellt einen `Style`-Struct aus einer HashMap, in der die Schlüssel die CSS-Property-Namen sind.
    /// Dabei wird versucht, die einzelnen Werte zu parsen.
    pub fn from_hashmap(map: &HashMap<String, String>) -> Self {
        let margin = parse_edge_from_hashmap(map, "margin");
        let padding = parse_edge_from_hashmap(map, "padding");
        let width = map.get("width")
            .and_then(|s| s.parse::<Dimension>().ok());
        let height = map.get("height")
            .and_then(|s| s.parse::<Dimension>().ok());
        let flex_direction = map.get("flex-direction")
            .and_then(|s| s.parse::<FlexDirection>().ok());

        let justify_content = map.get("justify-content")
            .and_then(|s| s.parse::<AlignContent>().ok());

        let align_items = map.get("align-items")
            .and_then(|s| s.parse::<AlignItems>().ok());

    
        let display = map.get("display").map(|s| {
            match s.as_str() {
                "block" => Display::Block,
                "flex" => Display::Flex,
                "grid" => Display::Grid,
                _ => Display::None,
            }
        });

        let font_size = map.get("font-size")
            .and_then(|s| s.parse::<f32>().ok());

        let background_color = map.get("background-color")
            .and_then(|s| {
                parse_color(s)
            });

        let color = map.get("color")
            .and_then(|s| {
                parse_color(s)
            });

        let gap = map.get("gap")
            .and_then(|s| s.parse::<Dimension>().ok());
        
        Self {
            margin,
            padding,
            width,
            height,
            flex_direction,
            display,
            font_size,
            background_color,
            color,
            gap,
            justify_content,
            align_items,
        }
    }

    pub fn from_str(value: &str) -> Self {
        let parsed_styles = parse_styles(&value);
        Self::from_hashmap(&parsed_styles)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_values_single() {
        let input = "10";
        let result = EdgeValues::from_str(input).unwrap();
        assert_eq!(result.top, 10.0);
        assert_eq!(result.right, 10.0);
        assert_eq!(result.bottom, 10.0);
        assert_eq!(result.left, 10.0);
    }

    #[test]
    fn test_edge_values_two() {
        let input = "10 20";
        let result = EdgeValues::from_str(input).unwrap();
        assert_eq!(result.top, 10.0);
        assert_eq!(result.right, 20.0);
        assert_eq!(result.bottom, 10.0);
        assert_eq!(result.left, 20.0);
    }

    #[test]
    fn test_edge_values_three() {
        let input = "10 20 30";
        let result = EdgeValues::from_str(input).unwrap();
        assert_eq!(result.top, 10.0);
        assert_eq!(result.right, 20.0);
        assert_eq!(result.bottom, 30.0);
        assert_eq!(result.left, 20.0);
    }

    #[test]
    fn test_edge_values_four() {
        let input = "10 20 30 40";
        let result = EdgeValues::from_str(input).unwrap();
        assert_eq!(result.top, 10.0);
        assert_eq!(result.right, 20.0);
        assert_eq!(result.bottom, 30.0);
        assert_eq!(result.left, 40.0);
    }

    #[test]
    fn test_dimension_auto() {
        let result = Dimension::from_str("auto").unwrap();
        assert_eq!(result, Dimension::Auto);
    }

    #[test]
    fn test_dimension_points() {
        let result = Dimension::from_str("100px").unwrap();
        assert_eq!(result, Dimension::Points(100.0));

        let result = Dimension::from_str("100pt").unwrap();
        assert_eq!(result, Dimension::Points(100.0));

        let result = Dimension::from_str("100").unwrap();
        assert_eq!(result, Dimension::Points(100.0));
    }

    #[test]
    fn test_dimension_percent() {
        let result = Dimension::from_str("50%").unwrap();
        assert_eq!(result, Dimension::Percent(50.0));
    }

    #[test]
    fn test_flex_direction_row() {
        let result = FlexDirection::from_str("row").unwrap();
        assert_eq!(result, FlexDirection::Row);
    }

    #[test]
    fn test_flex_direction_column() {
        let result = FlexDirection::from_str("column").unwrap();
        assert_eq!(result, FlexDirection::Column);
    }

    #[test]
    fn test_margin_and_padding_with_edges() {
        let mut map = HashMap::new();
        map.insert("margin-left".into(), "10px".into());
        map.insert("padding-top".into(), "28px".into());
        map.insert("padding-left".into(), "12px".into());

        let style = Style::from_hashmap(&map);

        // Test margin
        assert_eq!(
            style.margin.unwrap(),
            EdgeValues { left: 10.0, right: 0.0, bottom: 0.0, top: 0.0 }
        );

        // Test padding
        assert_eq!(
            style.padding.unwrap(),
            EdgeValues { left: 12.0, right: 0.0, bottom: 0.0, top: 28.0 }
        );
    }

    #[test]
    fn test_style_from_hashmap() {
        let mut map = HashMap::new();
        map.insert("margin".into(), "10px 20px".into());
        map.insert("padding".into(), "5 10 15 20".into());
        map.insert("width".into(), "100".into());
        map.insert("height".into(), "50%".into());
        map.insert("flex-direction".into(), "row".into());

        let style = Style::from_hashmap(&map);

        // Test margin
        assert_eq!(
            style.margin.unwrap(),
            EdgeValues { top: 10.0, right: 20.0, bottom: 10.0, left: 20.0 }
        );

        // Test padding
        assert_eq!(
            style.padding.unwrap(),
            EdgeValues { top: 5.0, right: 10.0, bottom: 15.0, left: 20.0 }
        );

        // Test width
        assert_eq!(style.width.unwrap(), Dimension::Points(100.0));

        // Test height
        assert_eq!(style.height.unwrap(), Dimension::Percent(50.0));

        // Test flex_direction
        assert_eq!(style.flex_direction.unwrap(), FlexDirection::Row);
    }
}