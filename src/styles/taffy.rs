use super::{AlignContent, AlignItems, Dimension, EdgeValues, FlexDirection, Style};

use taffy::style::{
    Dimension as TaffyDimension,
    FlexDirection as TaffyFlexDirection,
    Display,
    LengthPercentage,
    LengthPercentageAuto,
    Style as TaffyStyle,
};
use taffy::geometry::{self, Rect, Size};

impl Style {

    /// Konvertiert deinen eigenen Style in einen Taffy-Style.
    /// Dabei werden:
    /// - margin als Rect<LengthPercentageAuto> (als px) konvertiert
    /// - padding als Rect<LengthPercentage> (als px) konvertiert
    /// - width/height direkt in Taffys Dimension übernommen
    /// - flex_direction in Taffys FlexDirection gemappt
    pub fn to_taffy_style(&self) -> TaffyStyle {
        TaffyStyle {
            display: self.display.clone().map(Into::into).unwrap_or(Display::Block),
            flex_direction: self.flex_direction.clone()
                .map(|fd| match fd {
                    FlexDirection::Row => TaffyFlexDirection::Row,
                    FlexDirection::Column => TaffyFlexDirection::Column,
                })
                .unwrap_or(TaffyFlexDirection::Row),
            size: Size {
                width: self.width.clone().map(Into::into).unwrap_or(TaffyDimension::Auto),
                height: self.height.clone().map(Into::into).unwrap_or(TaffyDimension::Auto),
            },
            margin: self.margin
                .as_ref()
                .map(Into::into)
                .unwrap_or_else(|| Rect {
                    top: LengthPercentageAuto::Length(0.0),
                    right: LengthPercentageAuto::Length(0.0),
                    bottom: LengthPercentageAuto::Length(0.0),
                    left: LengthPercentageAuto::Length(0.0),
                }),
            padding: self.padding
                .as_ref()
                .map(Into::into)
                .unwrap_or_else(|| Rect {
                    top: LengthPercentage::Length(0.0),
                    right: LengthPercentage::Length(0.0),
                    bottom: LengthPercentage::Length(0.0),
                    left: LengthPercentage::Length(0.0),
                }),
            gap: self.gap.as_ref()
                .map(|d| {
                    let a: LengthPercentage = d.clone().into();
                    geometry::Size {
                        width: a,
                        height: a,
                    }
                }).unwrap_or(geometry::Size {
                    width: LengthPercentage::Length(0.0),
                    height: LengthPercentage::Length(0.0),
                }),

            align_items: self.align_items.clone()
                .map(Into::into),
            
            justify_content: self.justify_content.clone()
                .map(Into::into),
            
            ..Default::default()
        }
    }
}

impl From<AlignContent> for taffy::style::AlignContent {
    fn from(ac: AlignContent) -> Self {
        match ac {
            AlignContent::Start => taffy::style::AlignContent::Start,
            AlignContent::End => taffy::style::AlignContent::End,
            AlignContent::FlexStart => taffy::style::AlignContent::FlexStart,
            AlignContent::FlexEnd => taffy::style::AlignContent::FlexEnd,
            AlignContent::Center => taffy::style::AlignContent::Center,
            AlignContent::Stretch => taffy::style::AlignContent::Stretch,
            AlignContent::SpaceBetween => taffy::style::AlignContent::SpaceBetween,
            AlignContent::SpaceEvenly => taffy::style::AlignContent::SpaceEvenly,
            AlignContent::SpaceAround => taffy::style::AlignContent::SpaceAround,
        }
    }
}

impl From<AlignItems> for taffy::style::AlignItems {
    fn from(ai: AlignItems) -> Self {
        match ai {
            AlignItems::Start => taffy::style::AlignItems::Start,
            AlignItems::End => taffy::style::AlignItems::End,
            AlignItems::FlexStart => taffy::style::AlignItems::FlexStart,
            AlignItems::FlexEnd => taffy::style::AlignItems::FlexEnd,
            AlignItems::Center => taffy::style::AlignItems::Center,
            AlignItems::Baseline => taffy::style::AlignItems::Baseline,
            AlignItems::Stretch => taffy::style::AlignItems::Stretch,
        }
    }
}

/// Konvertiert unsere EdgeValues in ein Taffy Margin-Rect (LengthPercentageAuto).
impl From<&EdgeValues> for Rect<LengthPercentageAuto> {
    fn from(e: &EdgeValues) -> Self {
        Rect {
            top: LengthPercentageAuto::Length(e.top),
            right: LengthPercentageAuto::Length(e.right),
            bottom: LengthPercentageAuto::Length(e.bottom),
            left: LengthPercentageAuto::Length(e.left),
        }
    }
}

/// Konvertiert unsere EdgeValues in ein Taffy Padding-Rect (LengthPercentage).
impl From<&EdgeValues> for Rect<LengthPercentage> {
    fn from(e: &EdgeValues) -> Self {
        Rect {
            top: LengthPercentage::Length(e.top),
            right: LengthPercentage::Length(e.right),
            bottom: LengthPercentage::Length(e.bottom),
            left: LengthPercentage::Length(e.left),
        }
    }
}

/// Konvertierung unserer Dimension in Taffys Dimension.
impl From<Dimension> for TaffyDimension {
    fn from(dim: Dimension) -> Self {
        match dim {
            Dimension::Auto => TaffyDimension::Auto,
            Dimension::Points(val) => TaffyDimension::Length(val / 100.0),
            Dimension::Percent(val) => TaffyDimension::Percent(val / 100.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converting_edge_to_taffy() {
        let input = "10 11 12 20px";
        let result = EdgeValues::from_str(input).unwrap();
        
        let taffy_result: Rect<LengthPercentageAuto> = (&result).into();
        assert_eq!(taffy_result.top,    LengthPercentageAuto::Length(10.0));
        assert_eq!(taffy_result.right,  LengthPercentageAuto::Length(11.0));
        assert_eq!(taffy_result.bottom, LengthPercentageAuto::Length(12.0));
        assert_eq!(taffy_result.left,   LengthPercentageAuto::Length(20.0));
    }

    #[test]
    fn test_converting_dimension_percent_to_taffy() {
        let d = Dimension::Percent(100.0);
        
        let taffy_result: TaffyDimension = d.into();
        assert_eq!(taffy_result, TaffyDimension::Percent(100.0));
    }

    #[test]
    fn test_converting_dimension_auto_to_taffy() {
        let d = Dimension::Auto;
        
        let taffy_result: TaffyDimension = d.into();
        assert_eq!(taffy_result, TaffyDimension::Auto);
    }

    #[test]
    fn test_converting_dimension_point_to_taffy() {
        let d = Dimension::Points(100.0);
        
        let taffy_result: TaffyDimension = d.into();
        assert_eq!(taffy_result, TaffyDimension::Length(100.0));
    }

    #[test]
    fn text_convert_style_to_taffystyle() {

        let s = Style {
            width: Some(Dimension::Points(100.0)),
            height: Some(Dimension::Auto),
            margin: Some(EdgeValues::from_str("10 20 30 40px").unwrap()),
            padding: Some(EdgeValues::from_str("5 10 15 20px").unwrap()),
            flex_direction: Some(FlexDirection::Row),
            ..Default::default()
        };

        let taffy_style = s.to_taffy_style();

        assert_eq!(taffy_style.size.width, TaffyDimension::Length(100.0));
        assert_eq!(taffy_style.size.height, TaffyDimension::Auto);
        assert_eq!(taffy_style.margin.top, LengthPercentageAuto::Length(10.0));
        assert_eq!(taffy_style.margin.right, LengthPercentageAuto::Length(20.0));
        assert_eq!(taffy_style.margin.bottom, LengthPercentageAuto::Length(30.0));
        assert_eq!(taffy_style.margin.left, LengthPercentageAuto::Length(40.0));
        assert_eq!(taffy_style.padding.top, LengthPercentage::Length(5.0));
        assert_eq!(taffy_style.padding.right, LengthPercentage::Length(10.0));
        assert_eq!(taffy_style.padding.bottom, LengthPercentage::Length(15.0));
        assert_eq!(taffy_style.padding.left, LengthPercentage::Length(20.0));
        assert_eq!(taffy_style.flex_direction, TaffyFlexDirection::Row);

    }
}