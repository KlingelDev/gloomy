use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Style attributes for a text span.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    pub color: (f32, f32, f32, f32),
    pub font_size: Option<f32>,
    pub font_family: Option<String>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: (1.0, 1.0, 1.0, 1.0),
            font_size: None,
            font_family: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

/// A span of text with consistent styling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSpan {
    pub text: String,
    pub style: TextStyle,
}

/// Parsed rich text with formatting spans.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichText {
    pub spans: Vec<TextSpan>,
    pub is_plain: bool,
}

impl RichText {
    /// Parse text that may contain HTML-like markup.
    /// Returns plain text rendering if no markup detected.
    pub fn parse(text: &str, default_style: TextStyle) -> Self {
        if !Self::has_markup(text) {
            return Self::plain(text, default_style);
        }
        
        match MarkupParser::parse(text, default_style.clone()) {
            Ok(spans) => Self {
                spans,
                is_plain: false,
            },
            Err(e) => {
                eprintln!("Rich text parse error: {}. \
                          Rendering as plain text.", e);
                Self::plain(text, default_style)
            }
        }
    }
    
    /// Create plain text (no markup).
    pub fn plain(text: &str, style: TextStyle) -> Self {
        Self {
            spans: vec![TextSpan {
                text: text.to_string(),
                style,
            }],
            is_plain: true,
        }
    }
    
    /// Get plain text without formatting.
    pub fn to_plain(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }
    
    /// Quick check if text contains markup.
    pub fn has_markup(text: &str) -> bool {
        text.contains('<') && text.contains('>')
    }
    
    /// Measure total text dimensions (approximate).
    pub fn measure(&self, default_size: f32) -> (f32, f32) {
        let mut total_width = 0.0;
        let mut max_height = default_size;
        
        for span in &self.spans {
            let size = span.style.font_size.unwrap_or(default_size);
            let char_count = span.text.chars().count();
            
            // Rough estimate: average char width is 0.6 * font_size
            total_width += char_count as f32 * size * 0.6;
            max_height = max_height.max(size);
        }
        
        (total_width, max_height)
    }
}

/// HTML-like markup parser.
struct MarkupParser {
    input: Vec<char>,
    pos: usize,
    style_stack: Vec<TextStyle>,
}

impl MarkupParser {
    /// Parse HTML-like markup into text spans.
    pub fn parse(
        text: &str,
        base_style: TextStyle,
    ) -> Result<Vec<TextSpan>, String> {
        let mut parser = Self {
            input: text.chars().collect(),
            pos: 0,
            style_stack: vec![base_style],
        };
        
        parser.parse_content()
    }
    
    fn parse_content(&mut self) -> Result<Vec<TextSpan>, String> {
        let mut spans = Vec::new();
        let mut current_text = String::new();
        
        while self.pos < self.input.len() {
            if self.peek() == Some('<') {
                // Save current text if any
                if !current_text.is_empty() {
                    spans.push(TextSpan {
                        text: current_text.clone(),
                        style: self.current_style().clone(),
                    });
                    current_text.clear();
                }
                
                // Parse tag
                let tag = self.parse_tag()?;
                
                match tag {
                    Tag::Open(tag_type, attrs) => {
                        let mut new_style = self.current_style().clone();
                        self.apply_tag(&tag_type, &attrs, &mut new_style);
                        self.style_stack.push(new_style);
                    }
                    Tag::Close(_) => {
                        if self.style_stack.len() > 1 {
                            self.style_stack.pop();
                        }
                    }
                }
            } else {
                current_text.push(self.consume());
            }
        }
        
        // Save remaining text
        if !current_text.is_empty() {
            spans.push(TextSpan {
                text: current_text,
                style: self.current_style().clone(),
            });
        }
        
        Ok(spans)
    }
    
    fn parse_tag(&mut self) -> Result<Tag, String> {
        self.expect('<')?;
        
        // Check for closing tag
        let is_closing = self.peek() == Some('/');
        if is_closing {
            self.consume(); // consume '/'
        }
        
        // Parse tag name
        let tag_name = self.parse_identifier()?;
        
        // Parse attributes (only for opening tags)
        let attrs = if !is_closing {
            self.parse_tag_content(&tag_name)?
        } else {
            HashMap::new()
        };
        
        self.expect('>')?;
        
        let tag_type = match tag_name.to_lowercase().as_str() {
            "color" => TagType::Color,
            "size" => TagType::Size,
            "font" => TagType::Font,
            "bold" | "b" => TagType::Bold,
            "italic" | "i" => TagType::Italic,
            "underline" | "u" => TagType::Underline,
            "span" => TagType::Span,
            _ => {
                return Err(format!("Unknown tag: {}", tag_name));
            }
        };
        
        if is_closing {
            Ok(Tag::Close(tag_type))
        } else {
            Ok(Tag::Open(tag_type, attrs))
        }
    }
    
    fn parse_tag_content(
        &mut self,
        tag_name: &str,
    ) -> Result<HashMap<String, String>, String> {
        let mut attrs = HashMap::new();
        
        self.skip_whitespace();
        
        // Check for direct value (e.g., <color="#FF0000">)
        if self.peek() == Some('=') {
            self.consume(); // '='
            let value = self.parse_attribute_value()?;
            // Use tag name as implicit attribute key
            attrs.insert(tag_name.to_string(), value);
            self.skip_whitespace();
        }
        
        // Parse additional attributes
        while self.peek() != Some('>') && self.peek() != Some('/') {
            let key = self.parse_identifier()?;
            self.skip_whitespace();
            
            if self.peek() == Some('=') {
                self.consume(); // '='
                self.skip_whitespace();
                
                let value = self.parse_attribute_value()?;
                attrs.insert(key, value);
            } else {
                // Boolean attribute (e.g., bold)
                attrs.insert(key, "true".to_string());
            }
            
            self.skip_whitespace();
        }
        
        Ok(attrs)
    }
    
    fn parse_identifier(&mut self) -> Result<String, String> {
        let mut ident = String::new();
        
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                ident.push(self.consume());
            } else {
                break;
            }
        }
        
        if ident.is_empty() {
            Err("Expected identifier".to_string())
        } else {
            Ok(ident)
        }
    }
    
    fn parse_attribute_value(&mut self) -> Result<String, String> {
        let quote = self.peek();
        
        if quote == Some('"') || quote == Some('\'') {
            let quote_char = self.consume();
            let mut value = String::new();
            
            while self.peek() != Some(quote_char) {
                if self.peek().is_none() {
                    return Err("Unterminated string".to_string());
                }
                value.push(self.consume());
            }
            
            self.consume(); // closing quote
            Ok(value)
        } else {
            // Unquoted attribute value
            let mut value = String::new();
            while let Some(ch) = self.peek() {
                if ch.is_whitespace() || ch == '>' {
                    break;
                }
                value.push(self.consume());
            }
            Ok(value)
        }
    }
    
    fn apply_tag(
        &self,
        tag_type: &TagType,
        attrs: &HashMap<String, String>,
        style: &mut TextStyle,
    ) {
        match tag_type {
            TagType::Color => {
                // Check for "color" key or tag name
                if let Some(color_str) = attrs.get("color")
                    .or_else(|| attrs.get("c"))
                {
                    if let Ok(color) = parse_hex_color(color_str) {
                        style.color = color;
                    }
                }
            }
            TagType::Size => {
                // Check for "size" key or tag name
                if let Some(size_str) = attrs.get("size")
                    .or_else(|| attrs.get("s"))
                {
                    if let Ok(size) = size_str.parse::<f32>() {
                        style.font_size = Some(size);
                    }
                }
            }
            TagType::Font => {
                // Check for "font" key or tag name
                if let Some(font) = attrs.get("font")
                    .or_else(|| attrs.get("f"))
                {
                    style.font_family = Some(font.clone());
                }
            }
            TagType::Bold => {
                style.bold = true;
            }
            TagType::Italic => {
                style.italic = true;
            }
            TagType::Underline => {
                style.underline = true;
            }
            TagType::Span => {
                // Apply all attributes
                if let Some(color_str) = attrs.get("color") {
                    if let Ok(color) = parse_hex_color(color_str) {
                        style.color = color;
                    }
                }
                if let Some(size_str) = attrs.get("size") {
                    if let Ok(size) = size_str.parse::<f32>() {
                        style.font_size = Some(size);
                    }
                }
                if let Some(font) = attrs.get("font") {
                    style.font_family = Some(font.clone());
                }
                if attrs.get("bold").is_some() {
                    style.bold = true;
                }
                if attrs.get("italic").is_some() {
                    style.italic = true;
                }
                if attrs.get("underline").is_some() {
                    style.underline = true;
                }
            }
        }
    }
    
    fn current_style(&self) -> &TextStyle {
        self.style_stack.last().unwrap()
    }
    
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }
    
    fn consume(&mut self) -> char {
        let ch = self.input[self.pos];
        self.pos += 1;
        ch
    }
    
    fn expect(&mut self, expected: char) -> Result<(), String> {
        if self.peek() == Some(expected) {
            self.consume();
            Ok(())
        } else {
            Err(format!(
                "Expected '{}', found {:?}",
                expected,
                self.peek()
            ))
        }
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }
}

#[derive(Debug)]
enum Tag {
    Open(TagType, HashMap<String, String>),
    Close(TagType),
}

#[derive(Debug, Clone, PartialEq)]
enum TagType {
    Color,
    Size,
    Font,
    Bold,
    Italic,
    Underline,
    Span,
}

/// Parse hex color string to RGBA floats (0.0-1.0).
pub fn parse_hex_color(
    hex: &str,
) -> Result<(f32, f32, f32, f32), String> {
    let hex = hex.trim_start_matches('#');
    
    let (r, g, b, a) = match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| "Invalid red component")?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| "Invalid green component")?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| "Invalid blue component")?;
            (r, g, b, 255u8)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| "Invalid red component")?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| "Invalid green component")?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| "Invalid blue component")?;
            let a = u8::from_str_radix(&hex[6..8], 16)
                .map_err(|_| "Invalid alpha component")?;
            (r, g, b, a)
        }
        _ => {
            return Err(format!(
                "Invalid hex color: {}. Expected 6 or 8 characters",
                hex
            ));
        }
    };
    
    Ok((
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_has_markup() {
        assert!(RichText::has_markup("<color=\"#FF0000\">Text</color>"));
        assert!(!RichText::has_markup("Plain text"));
    }
    
    #[test]
    fn test_parse_color_tag() {
        let rt = RichText::parse(
            "<color=\"#FF0000\">Red</color>",
            TextStyle::default(),
        );
        assert_eq!(rt.spans.len(), 1);
        assert_eq!(rt.spans[0].text, "Red");
        assert_eq!(rt.spans[0].style.color, (1.0, 0.0, 0.0, 1.0));
    }
    
    #[test]
    fn test_parse_bold_tag() {
        let rt = RichText::parse("<bold>Bold</bold>", TextStyle::default());
        assert_eq!(rt.spans.len(), 1);
        assert!(rt.spans[0].style.bold);
    }
    
    #[test]
    fn test_parse_nested_tags() {
        let rt = RichText::parse(
            "<bold><color=\"#FF0000\">Bold Red</color></bold>",
            TextStyle::default(),
        );
        assert_eq!(rt.spans.len(), 1);
        assert!(rt.spans[0].style.bold);
        assert_eq!(rt.spans[0].style.color, (1.0, 0.0, 0.0, 1.0));
    }
    
    #[test]
    fn test_parse_span_tag() {
        let rt = RichText::parse(
            "<span color=\"#00FF00\" size=\"24\" bold>Styled</span>",
            TextStyle::default(),
        );
        assert_eq!(rt.spans.len(), 1);
        assert_eq!(rt.spans[0].style.color, (0.0, 1.0, 0.0, 1.0));
        assert_eq!(rt.spans[0].style.font_size, Some(24.0));
        assert!(rt.spans[0].style.bold);
    }
    
    #[test]
    fn test_mixed_plain_and_markup() {
        let rt = RichText::parse(
            "Plain <bold>Bold</bold> Plain",
            TextStyle::default(),
        );
        assert_eq!(rt.spans.len(), 3);
        assert_eq!(rt.spans[0].text, "Plain ");
        assert!(!rt.spans[0].style.bold);
        assert_eq!(rt.spans[1].text, "Bold");
        assert!(rt.spans[1].style.bold);
        assert_eq!(rt.spans[2].text, " Plain");
        assert!(!rt.spans[2].style.bold);
    }
    
    #[test]
    fn test_parse_hex_color() {
        assert_eq!(
            parse_hex_color("#FF0000").unwrap(),
            (1.0, 0.0, 0.0, 1.0)
        );
        assert_eq!(
            parse_hex_color("#00FF00FF").unwrap(),
            (0.0, 1.0, 0.0, 1.0)
        );
        assert_eq!(
            parse_hex_color("0000FF").unwrap(),
            (0.0, 0.0, 1.0, 1.0)
        );
    }
}
