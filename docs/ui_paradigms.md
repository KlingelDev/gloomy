# UI Architecture Models

This document outlines common user interface architectures to provide context for Gloomy's design decisions.

## 1. Retained Mode (Object-Oriented)
**Examples**: DOM (HTML), Qt Widgets, Swing, Windows Forms, JavaFX.

In retained mode, the UI library maintains a persistent tree of objects (widgets/elements) representing the UI.
- **State**: Stored within the widget objects themselves (e.g., `button.setText("Click me")`).
- **Updates**: The application acts on these objects to change their properties. The system automatically repaints changes.
- **Events**: Event listeners (callbacks) are attached to specific objects.
- **Pros**: Easy to understand for static UIs, highly optimized rendering (only repaint what changed).
- **Cons**: Synchronization hell; keeping the UI state in sync with app logic requires manual updates or complex observing patterns.

## 2. Immediate Mode (IMGUI)
**Examples**: Dear ImGui, egui.

In immediate mode, the UI tree is rebuilt every single frame by running the code that describes it.
- **State**: The UI toolkit holds minimal state (e.g., "window is open", "hot widget ID"). The app holds all domain state.
- **Updates**: You don't "update" a label; you just call `label("New text")` in the next frame.
- **Events**: Input is handled during the draw call (e.g., `if button("Click").clicked() { ... }`).
- **Pros**: No state synchronization issues, extremely fast for prototyping and dynamic tools.
- **Cons**: Harder to layout complex static content, heavy CPU usage (rebuilding every frame), difficult to do animations or complex async layouts.

## 3. The Elm Architecture (Model-View-Update / MVU)
**Examples**: Elm, Iced (Rust), Relm4, Tea (Go).

A purely functional approach based on unidirectional data flow.
- **Model**: A single immutable data structure representing the entire application state.
- **View**: A pure function `Model -> UI` that generates a lightweight description of the UI (virtual DOM).
- **Update**: A pure function `Message + Model -> Model` that handles events and transitions state.
- **Pros**: Deterministic, easy to test, no synchronization bugs.
- **Cons**: Can be verbose (events for everything), performance overhead of diffing virtual DOMs.

## 4. Reactive / Declarative (Signal-based)
**Examples**: React, SolidJS, Sycamore, Xilem, Leptos.

Similar to MVU but often with finer-grained reactivity.
- **Declarative**: You describe *what* the UI should look like based on state, not *how* to change it.
- **Signals/Hooks**: State is wrapped in reactive primitives. When a signal changes, only the parts of the UI depending on it are re-computed.
- **Pros**: Best of both worlds—declarative (like MVU) but performant (fine-grained updates).
- **Cons**: Learning curve for reactivity (hooks, dependencies, ownership).

---

## Where Gloomy Fits

Gloomy currently employs a **Hybrid Retained/Immediate** model:

- **Structure (Retained)**: You build a `Widget` tree (structs like `Container`, `Label`) that persists in memory (`ui_root`).
- **Layout & Rendering (Immediate-style)**:
    - The `compute_layout` function is called every frame (or on change) to recalculate positions.
    - The `render_ui` function traverses the entire tree every frame to generate GPU primitives.
- **State**: State is largely external (`AppState`), but widgets are mutable objects that can be modified directly (Retained style).

### Gloomy's "Event Based" Nature
Gloomy uses a classic event loop (`winit`). Events are captured at the window level and propagated.
- **Input**: Passed into `InteractionState`.
- **Logic**: Users write callbacks (`on_mouse_input`, `on_draw`) that manually mutate the external state or the widget tree.


This is closest to a low-level **Retained Mode** engine where the user is responsible for the manual "game loop" of updating the tree.

### Strategy: Avoiding Synchronization Hell

To effectively work with Gloomy's hybrid model and avoiding sync issues, you must follow the **Unidirectional Data Flow** pattern (similar to Elm/React):

1.  **App State is the Only Truth**: Never treat the `Widget` tree as a storage for data. 
    - **Wrong**: `widget.text_input.text = "hello"` then reading it back later.
    - **Right**: `app_state.username = "hello"`, then `Widget::TextInput { text: app_state.username, ... }` is generated from it.

2.  **Events Update App State, Not Widgets**: When an input event occurs, do *not* mutate the widget tree directly. Mutate the `AppState`.
    - The next frame's `render` (or `make_ui`) logic will naturally read the new state and produce the correct widget tree.

3.  **Use Stable IDs for Ephemeral State**: Gloomy's internal `InteractionState` handles UI-specific state like focus, scroll position, and hover. This relies on stable Widget IDs.
    - **Requirement**: Always assign stable `id`s to interactive widgets (`TextInput`, `Scrollable` containers).

#### Example Pattern

```rust
// 1. Update (Event Handling)
fn on_input(state: &mut AppState, event: Event) {
   if let Event::Text(c) = event {
       state.username.push(c); // Mutate SOURCE of truth
   }
}

// 2. View (Render)
fn view(state: &AppState) -> Widget {
    Widget::TextInput {
        id: "username_field".into(), // Stable ID for InteractionState
        text: state.username.clone(), // Derived from source
    }
}
```
