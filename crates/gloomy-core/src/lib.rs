//! Gloomy Core - GPU rendering primitives for the gloomy UI library.
//!
//! This crate provides the foundational rendering components:
//! - SDF-based primitive rendering (rectangles, circles, lines)
//! - TTF text rendering
//! - Container-based layout system
//! - RON-based declarative UI definitions
//! - Flexbox-style layout engine
//! - Interactive UI elements (buttons)
//! - GPU context management

pub mod container;
pub mod interaction;
pub mod image_renderer;
pub mod layout;
pub mod layout_engine;
pub mod primitives;
pub mod rect;
pub mod renderer;
pub mod text;
pub mod texture;
pub mod ui;
pub mod widget;
pub mod svg_loader;
pub mod theme;
pub mod style;
pub mod style_context;
pub mod data_source;
pub mod datagrid;
pub mod tree;
pub mod validation;
pub mod rich_text;
pub mod widget_state;
pub mod kpi;

pub use container::Container;
pub use glam::{Vec2, Vec4};
pub use interaction::InteractionState;
pub use layout::{Align, Direction, Justify, Layout};
pub use layout_engine::compute_layout;
pub use primitives::{Instance, PrimitiveRenderer};
pub use rect::Rect;
pub use renderer::GloomyRenderer;
pub use text::TextRenderer;
pub use ui::{
  hit_test, load_ui, parse_ui, render_ui, RenderContext,
};
pub use widget::{Widget, WidgetBounds};
pub use theme::{Theme, ColorPalette};
pub use style::{GlobalStyle, BoxStyle, ButtonStyle, TextInputStyle, Shadow, Gradient, Border, BorderStyle};
pub use style_context::StyleContext;
pub use data_source::{DataSource, CellValue, VecDataSource};
pub use datagrid::{ColumnDef, ColumnWidth, DataGrid, DataGridStyle, SelectionMode, SortDirection};
pub use kpi::{KpiCard, KpiCardStyle, KpiTrend, TrendDirection};
