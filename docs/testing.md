# Testing Gloomy Applications

This document outlines how to test Gloomy UIs using the
`gloomy-driver` crate for headless automation.

## Approaches

### 1. Unit Tests (Logic)
Test pure logic functions and state transitions in your
application code.
- **Scope**: `AppState` mutations, data processing,
  validation logic.
- **Location**: Standard `#[test]` blocks in your modules.

### 2. Integration Tests (Headless Driver)
Use `gloomy-driver` to simulate user interaction with the
widget tree without spawning a window. This verifies that
your UI logic (widget construction -> interaction -> state
update) works correctly.

**The `GloomyDriver`:**
- Wraps a `Widget` tree.
- Calculates layout (so hit-testing works).
- Provides methods to find widgets, simulate input, and
  query widget state.

**Example:**

```rust
#[test]
fn test_login_flow() {
    let ui = build_ui(&AppState::default());
    let mut driver =
        GloomyDriver::new(ui, 800.0, 600.0);

    // Type into text inputs directly.
    driver.set_text("username", "admin").unwrap();
    driver.set_text("password", "secret").unwrap();

    // Verify the input took effect.
    assert_eq!(
        driver.get_text("username"),
        Some("admin".to_string()),
    );

    // Click the login button.
    let action = driver.click("login_btn");
    assert_eq!(
        action,
        Some("login_submit".to_string()),
    );
}
```

### 3. Input Simulation

The driver can set widget values directly, calling
`relayout()` automatically so subsequent queries reflect
the new state.

| Method | Widget types | Notes |
|--------|-------------|-------|
| `set_text(id, text)` | `TextInput`, `Autocomplete` | Sets `value` |
| `set_number(id, val)` | `NumberInput` | Clamps to `min`/`max` |
| `set_slider(id, val)` | `Slider` | Clamps to `min`/`max` |
| `toggle(id)` | `Checkbox`, `ToggleSwitch` | Returns new `bool` |
| `select(id, idx)` | `Dropdown`, `ListView` | Errors on out-of-bounds |
| `select_tab(id, idx)` | `Tab` | Errors on out-of-bounds |

All methods return `Result<()>` (or `Result<bool>` for
`toggle`) and error if the widget is not found or is the
wrong type.

**Example — form interaction:**

```rust
let mut driver = GloomyDriver::new(form_ui, 800.0, 600.0);

driver.set_text("name_input", "Alice").unwrap();
driver.set_number("age_input", 30.0).unwrap();
driver.set_slider("volume", 0.8).unwrap();
driver.toggle("agree_checkbox").unwrap();
driver.select("country_dropdown", 2).unwrap();
driver.select_tab("settings_tabs", 1).unwrap();
```

### 4. State Querying

Read current widget state without mutating the tree.

| Method | Widget types | Returns |
|--------|-------------|---------|
| `get_text(id)` | `TextInput`, `Autocomplete`, `Label`, `Button` | `Option<String>` |
| `get_number(id)` | `NumberInput`, `Slider` | `Option<f64>` |
| `is_checked(id)` | `Checkbox`, `ToggleSwitch` | `Option<bool>` |
| `get_selected(id)` | `Dropdown`, `ListView`, `Tab` | `Option<usize>` |

Returns `None` if the widget is not found or is an
unsupported type.

**Example — assertions:**

```rust
assert_eq!(driver.get_text("name"), Some("Alice".into()));
assert_eq!(driver.get_number("age"), Some(30.0));
assert_eq!(driver.is_checked("agree"), Some(true));
assert_eq!(driver.get_selected("country"), Some(2));
```

### 5. Visual Snapshot Testing

Render the widget tree to an image and compare against a
stored golden reference.

**`snapshot_test()`** — returns a `DiffReport` with pixel
counts and pass/fail status:

```rust
let mut driver = GloomyDriver::new(ui, 800.0, 600.0);
driver.init_renderer(false)?;

let report = driver.snapshot_test(
    "dashboard",
    "tests/snapshots",
    None,
)?;
assert!(report.passed);
```

**`assert_screenshot()`** — convenience wrapper that errors
on any mismatch beyond the given tolerance:

```rust
assert_screenshot(
    &mut driver,
    "login_form",
    "tests/snapshots",
    2, // per-channel tolerance (0-255)
)?;
```

On the first run (no golden exists), the rendered image is
saved as the golden reference. On subsequent runs, the
actual render is compared pixel-by-pixel. If the images
differ, `_actual.png` and `_diff.png` files are saved next
to the golden for inspection.

## Best Practices

1.  **Stable IDs**: Assign unique, stable `id`s to all
    interactive widgets (`Button`, `TextInput`,
    `Container`s used for navigation) to make them findable
    by the driver.
2.  **Decouple Logic**: Keep your `build_ui` and `update`
    logic separate from the `winit` event loop so they can
    be called by tests.
3.  **Query after mutation**: Use `get_text`, `get_number`,
    etc. to verify that input simulation had the expected
    effect before proceeding with further interactions.
