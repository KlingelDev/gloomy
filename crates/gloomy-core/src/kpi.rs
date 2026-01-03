use serde::{Deserialize, Serialize};
use crate::widget::{Color, WidgetBounds};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KpiCardStyle {
    pub background: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub corner_radius: f32,
    pub label_color: Color,
    pub label_size: f32,
    pub value_color: Color,
    pub value_size: f32,
    pub trend_up_color: Color,
    pub trend_down_color: Color,
    pub trend_neutral_color: Color,
}

impl Default for KpiCardStyle {
    fn default() -> Self {
        Self {
            background: (0.15, 0.15, 0.2, 1.0),
            border_color: (0.3, 0.3, 0.3, 1.0),
            border_width: 1.0,
            corner_radius: 8.0,
            label_color: (0.7, 0.7, 0.7, 1.0),
            label_size: 14.0,
            value_color: (1.0, 1.0, 1.0, 1.0),
            value_size: 24.0,
            trend_up_color: (0.2, 0.8, 0.2, 1.0),
            trend_down_color: (0.8, 0.2, 0.2, 1.0),
            trend_neutral_color: (0.6, 0.6, 0.6, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum TrendDirection {
    Up,
    Down,
    Neutral,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KpiTrend {
    pub direction: TrendDirection,
    pub value: String, // e.g. "+12%"
}

/// A KPI Card widget for displaying a key metric.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KpiCard {
    #[serde(default)]
    pub title: String,
    
    #[serde(default)]
    pub value: String,
    
    #[serde(default)]
    pub trend: Option<KpiTrend>,
    
    #[serde(default)]
    pub style: KpiCardStyle,
    
    #[serde(default)]
    pub bounds: WidgetBounds,
}
