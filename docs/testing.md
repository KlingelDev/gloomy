# Testing Gloomy Applications

This document outlines how to test Gloomy UIs using the `gloomy-driver` crate for headless automation.

## Approaches

### 1. Unit Tests (Logic)
Test pure logic functions and state transitions in your application code.
- **Scope**: `AppState` mutations, data processing, validation logic.
- **Location**: Standard `#[test]` blocks in your modules.

### 2. Integration Tests (Headless Driver)
Use `gloomy-driver` to simulate user interaction with the widget tree without spawning a window. This verifies that your UI logic (widget construction -> interaction -> state update) works correctly.

**The `GloomyDriver`:**
- Wraps a `Widget` tree.
- Calculates layout (so hit-testing works).
- Provides methods to find widgets and simulate events.

**Example:**

```rust
#[test]
fn test_login_flow() {
    // 1. Setup App State
    let mut state = AppState::default();
    
    // 2. Build UI
    let ui = build_ui(&state);
    
    // 3. Create Driver
    let mut driver = GloomyDriver::new(ui, 800.0, 600.0);
    
    // 4. Interact
    // Simulate typing (update state directly as driver doesn't handle keyboard events yet)
    state.username = "admin".to_string();
    
    // Re-build UI to reflect state change
    let ui = build_ui(&state);
    let mut driver = GloomyDriver::new(ui, 800.0, 600.0);
    
    // Click button
    let action = driver.click("login_btn");
    assert_eq!(action, Some("login_submit".to_string()));
    
    // 5. Handle Action
    if let Some(act) = action {
        update_state(&mut state, act);
    }
    
    // 6. Verify Result
    let ui = build_ui(&state);
    let driver = GloomyDriver::new(ui, 800.0, 600.0);
    
    assert!(driver.find("success_message").is_some());
}
```

### 3. Visual Tests (Future)
Future plans include visual regression testing by rendering frames to images and comparing them against "golden" snapshots.

## Best Practices

1.  **Stable IDs**: Assign unique, stable `id`s to all interactive widgets (`Button`, `TextInput`, `Container`s used for navigation) to make them findable by the driver.
2.  **Decouple Logic**: Keep your `build_ui` and `update` logic separate from the `winit` event loop so they can be called by tests.