//! RON export and import utilities.

use gloomy_core::widget::Widget;
use ron::error::SpannedError;

/// Exports a widget tree to RON format with pretty printing.
pub fn export_to_ron(widget: &Widget) -> Result<String, ron::Error> {
    let config = ron::ser::PrettyConfig::new()
        .depth_limit(20)
        .indentor("    ".to_string());
    
    ron::ser::to_string_pretty(widget, config)
}

/// Imports a widget tree from RON format.
pub fn import_from_ron(ron_str: &str) -> Result<Widget, SpannedError> {
    ron::from_str(ron_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_roundtrip() {
        let widget = Widget::container();
        let ron = export_to_ron(&widget).unwrap();
        let parsed = import_from_ron(&ron).unwrap();
        let ron2 = export_to_ron(&parsed).unwrap();
        assert_eq!(ron, ron2);
    }
}
