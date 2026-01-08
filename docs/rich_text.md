# Rich Text System

Gloomy supports a universal rich text system across most widgets (Labels, Buttons, DataGrid cells, Tree nodes, etc.). It uses an HTML-like markup syntax to style text segments.

## Syntax

Rich text is defined using XML-like tags. Any text not within tags is rendered with the widget's default style.

```xml
This is <bold>bold</bold> and this is <color="#FF0000">red</color>.
```

## Supported Tags

### Style Modifiers

- `<bold>` or `<b>`: Makes text **bold**.
- `<italic>` or `<i>`: Makes text *italic*.
- `<underline>` or `<u>`: Adds an <u>underline</u>.

### Color

- `<color="#RRGGBB">` or `<c="#RRGGBB">`: Changes text color.
- `<color="#RRGGBBAA">`: Changes text color with alpha.

### Size

- `<size="24">` or `<s="24">`: Changes font size in pixels.

### Font Family

- `<font="Arial">` or `<f="Arial">`: Changes the font family (must be loaded).

### Generic Span

The `<span>` tag allows combining multiple attributes.

```xml
<span color="#00FF00" size="20" bold>Big Green Bold Text</span>
```

Supported attributes on `span`:
- `color` or `c`
- `size` or `s`
- `font` or `f`
- `bold`, `italic`, `underline` (boolean attributes)

## Parsing

The system uses `RichText::parse(text, default_style)` internally. If parsing fails (e.g., malformed tags), it falls back to rendering the raw text string.

## Performance

The rich text parser is efficient, but heavy use of complex markup in large DataGrids (thousands of rows) may have a small impact. It is generally negligible for typical UI usage.
