//! Application struct managing the event loop and windows.

use crate::GloomyWindow;
use std::collections::HashMap;
use std::sync::Arc;
use winit::event::{ElementState, Event, WindowEvent, MouseButton};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::{WindowBuilder, WindowId};

/// Context passed to the draw callback with GPU resources.
pub struct DrawContext<'a> {
  pub device: &'a wgpu::Device,
  pub queue: &'a wgpu::Queue,
}

/// Callback for drawing a window frame.
pub type DrawFn = Box<dyn FnMut(&mut GloomyWindow, &DrawContext)>;

/// Callback for mouse move events.
pub type CursorMoveFn = Box<dyn FnMut(&mut GloomyWindow, f32, f32)>;

/// Callback for mouse clicks.
pub type MouseInputFn = Box<dyn FnMut(&mut GloomyWindow, ElementState, MouseButton)>;

/// Callback for keyboard input.
pub type KeyboardInputFn = Box<dyn FnMut(&mut GloomyWindow, winit::event::KeyEvent)>;

/// Callback for mouse wheel (scroll).
pub type ScrollFn = Box<dyn FnMut(&mut GloomyWindow, winit::event::MouseScrollDelta, winit::event::TouchPhase)>;

/// Callback for modifiers changed.
pub type ModifiersChangedFn = Box<dyn FnMut(&mut GloomyWindow, winit::event::Modifiers)>;

/// A gloomy application managing multiple windows.
pub struct GloomyApp {
  draw_fn: Option<DrawFn>,
  cursor_move_fn: Option<CursorMoveFn>,
  mouse_input_fn: Option<MouseInputFn>,
  keyboard_input_fn: Option<KeyboardInputFn>,
  scroll_fn: Option<ScrollFn>,
  modifiers_changed_fn: Option<ModifiersChangedFn>,
}

/// Runtime state during event loop.
struct AppState {
  _instance: wgpu::Instance,
  _adapter: wgpu::Adapter,
  device: wgpu::Device,
  queue: wgpu::Queue,
  windows: HashMap<winit::window::WindowId, GloomyWindow>,
}

impl GloomyApp {
  /// Creates a new application.
  pub fn new() -> Self {
    Self { 
      draw_fn: None,
      cursor_move_fn: None,
      mouse_input_fn: None,
      keyboard_input_fn: None,
      scroll_fn: None,
      modifiers_changed_fn: None,
    }
  }

  // ... (existing methods)

  /// Sets the keyboard input callback.
  pub fn on_keyboard_input<F>(mut self, f: F) -> Self
  where
    F: FnMut(&mut GloomyWindow, winit::event::KeyEvent) + 'static,
  {
      self.keyboard_input_fn = Some(Box::new(f));
      self
  }

  /// Sets the scroll callback.
  pub fn on_scroll<F>(mut self, f: F) -> Self
  where
    F: FnMut(&mut GloomyWindow, winit::event::MouseScrollDelta, winit::event::TouchPhase) + 'static,
  {
      self.scroll_fn = Some(Box::new(f));
      self
  }

  // ... (run loop)


  /// Sets the draw callback for windows.
  pub fn on_draw<F>(mut self, f: F) -> Self
  where
    F: FnMut(&mut GloomyWindow, &DrawContext) + 'static,
  {
    self.draw_fn = Some(Box::new(f));
    self
  }

  /// Sets the cursor move callback.
  pub fn on_cursor_move<F>(mut self, f: F) -> Self
  where
    F: FnMut(&mut GloomyWindow, f32, f32) + 'static,
  {
      self.cursor_move_fn = Some(Box::new(f));
      self
  }

  /// Sets the mouse input callback.
  pub fn on_mouse_input<F>(mut self, f: F) -> Self
  where 
    F: FnMut(&mut GloomyWindow, ElementState, MouseButton) + 'static,
  {
      self.mouse_input_fn = Some(Box::new(f));
      self
  }

  /// Sets the modifiers changed callback.
  pub fn on_modifiers_changed<F>(mut self, f: F) -> Self
  where
    F: FnMut(&mut GloomyWindow, winit::event::Modifiers) + 'static,
  {
      self.modifiers_changed_fn = Some(Box::new(f));
      self
  }

  /// Runs the application event loop.
  pub fn run(mut self) -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;

    // Create initial window
    let window = Arc::new(
      WindowBuilder::new()
        .with_title("Gloomy")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)?,
    );

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::all(),
      ..Default::default()
    });

    let surface = instance.create_surface(window.clone())?;

    let adapter = pollster::block_on(instance.request_adapter(
      &wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
      },
    ))
    .ok_or_else(|| anyhow::anyhow!("Failed to find GPU adapter"))?;

    let (device, queue) = pollster::block_on(adapter.request_device(
      &wgpu::DeviceDescriptor {
        label: Some("GloomyDevice"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
      },
      None,
    ))?;

    let gloomy_window =
      GloomyWindow::new(window, &instance, &adapter, &device)?;

    let mut state = AppState {
      _instance: instance,
      _adapter: adapter,
      device,
      queue,
      windows: HashMap::new(),
    };

    let window_id = gloomy_window.id();
    state.windows.insert(window_id, gloomy_window);

    #[allow(deprecated)]
    event_loop.run(move |event, elwt| match event {
      Event::WindowEvent { window_id, event } => {
        self.handle_window_event(&mut state, window_id, event, elwt);
      }
      Event::AboutToWait => {
        for win in state.windows.values() {
          win.window.request_redraw();
        }
      }
      _ => {}
    })?;

    Ok(())
  }

  fn handle_window_event(
    &mut self,
    state: &mut AppState,
    window_id: WindowId,
    event: WindowEvent,
    elwt: &winit::event_loop::EventLoopWindowTarget<()>,
  ) {
    match event {
      WindowEvent::CloseRequested => {
        state.windows.remove(&window_id);
        if state.windows.is_empty() {
          elwt.exit();
        }
      }

      WindowEvent::Resized(size) => {
        if let Some(win) = state.windows.get_mut(&window_id) {
          win.resize(&state.device, &state.queue, size.width, size.height);
        }
      }

      WindowEvent::CursorMoved { position, .. } => {
        if let Some(win) = state.windows.get_mut(&window_id) {
            if let Some(cb) = self.cursor_move_fn.as_mut() {
                cb(win, position.x as f32, position.y as f32);
            }
        }
      }

      WindowEvent::MouseInput { state: element_state, button, .. } => {
          if let Some(win) = state.windows.get_mut(&window_id) {
              if let Some(cb) = self.mouse_input_fn.as_mut() {
                  cb(win, element_state, button);
              }
          }
      }

      WindowEvent::MouseWheel { delta, phase, .. } => {
        if let Some(win) = state.windows.get_mut(&window_id) {
            if let Some(cb) = self.scroll_fn.as_mut() {
                cb(win, delta, phase);
            }
        }
      }

      WindowEvent::RedrawRequested => {
        if let Some(win) = state.windows.get_mut(&window_id) {
          if let Some(draw_fn) = self.draw_fn.as_mut() {
            let ctx =
              DrawContext { device: &state.device, queue: &state.queue };
            draw_fn(win, &ctx);
          }

          if let Err(e) = win.render(&state.device, &state.queue) {
            log::error!("Render error: {:?}", e);
          }
        }
      }

      WindowEvent::KeyboardInput { event, .. } => {
        if let Some(win) = state.windows.get_mut(&window_id) {
             if let Some(cb) = self.keyboard_input_fn.as_mut() {
                 cb(win, event.clone());
             }
        }
        
        if event.state.is_pressed() {
          match &event.logical_key {
            Key::Character(c) if c.as_str() == "q" => {
               elwt.exit();
            }
            Key::Named(NamedKey::Escape) => {
              elwt.exit();
            }
            _ => {
              log::debug!("Key pressed: {:?}", event.logical_key);
            }
          }
        }
      }

      WindowEvent::ModifiersChanged(modifiers) => {
        if let Some(win) = state.windows.get_mut(&window_id) {
             if let Some(cb) = self.modifiers_changed_fn.as_mut() {
                 cb(win, modifiers);
             }
        }
      }

      _ => {}
    }
  }
}

impl Default for GloomyApp {
  fn default() -> Self {
    Self::new()
  }
}
