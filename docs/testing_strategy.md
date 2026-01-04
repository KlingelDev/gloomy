# Gloomy Testing Strategy

## Current State
- **Unit Tests**: 0
- **Integration Tests**: 0
- **Manual Verification**: Running examples (`cargo run --example dashboard_demo`).

## Goal
To enable confident refactoring and ensure stability through automated testing of:
1.  **Layout Logic**: "Does this container actually size to 50%?"
2.  **Interaction**: "Does clicking this button update the state?"
3.  **Visuals**: "Did we break the border rendering?"

## Proposed Layers

### 1. Pure Unit Tests (Logic)
The `gloomy-core` logic is largely separable from the GPU. We can test `layout_engine`, `validation`, and data structures directly.

**Example Plan:**
- Create `crates/gloomy-core/src/layout_engine_test.rs`.
- Construct `Widget` trees programmatically.
- Run `compute_layout`.
- Assert on `WidgetBounds`.

### 2. Headless Driver (Behavior / Integration)
To test "UI behavior" without a physical window, we use the **`gloomy-driver`** crate. This decouples the "Application" from `winit`.

**Architecture:**
- **`GloomyDriver` (crate: `gloomy-driver`)**: A wrapper around `Widget` tree + State, without `winit` or `wgpu`.
- **API**:
    - `driver.find("login_button").click()`
    - `driver.find("username_input").type_text("admin")`
    - `driver.assert_text("status_label", "Login Successful")`

**Mocking Time/Inputs:**
- The driver will manually pump the `InteractionState`.
- It will manually trigger `on_update` callbacks.

### 3. Visual Regression (Golden Tests)
For rendering correctness, we cannot rely on checking struct fields. We need **Snapshot Testing**.

**Strategy:**
- Use software rendering (via `tiny-skia` or `wgpu` with software adapter) to render a frame to a buffer.
- Compare the buffer against a stored "Golden" PNG.
- **Tooling**: `insta` crate for snapshot management, `image` crate for comparison.

## Roadmap to "Get There"

1.  **Phase 1: Foundation (Unit)**
    - Add `#[cfg(test)]` modules to `gloomy-core`.
    - Write tests for `Layout`, specifically corner cases of Flexbox/Grid.

2.  **Phase 2: The Driver (Automation)**
    - Implement `GloomyTestContext` (the Headless Driver).
    - Implement `find_widget_by_id`.
    - Implement `dispatch_click`, `dispatch_key`.

3.  **Phase 3: Visuals**
    - Set up a CI workflow that runs `cargo test`.
    - Add a "Goldens" folder.
    - Write a harness that renders specific examples to PNGs and diffs them.
