use gloomy_core::{
    widget::{Widget, WidgetBounds},
    layout_engine::compute_layout,
};
use std::time::Instant;

fn main() {
    println!("=== Layout Caching Benchmark ===");

    // 1. Create a deep/heavy tree
    let mut root = Widget::container();
    let mut current = &mut root;
    
    // Create simple hierarchy depth 1000
    // We need to properly construct it. Recursion is easier.
    
    // Construct tree
    let mut children = Vec::new();
    for i in 0..10_000 {
        children.push(Widget::label(format!("Item {}", i)));
    }
    
    if let Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None), children: c, .. } = &mut root {
        *c = children;
    }

    // 2. Initial Layout (Cold Cache)
    let start = Instant::now();
    compute_layout(&mut root, 0.0, 0.0, 1920.0, 1080.0);
    let cold_time = start.elapsed();
    println!("Cold Layout (10k items): {:?}", cold_time);

    // 3. Second Layout (Warm Cache) - Should be near zero
    let start = Instant::now();
    compute_layout(&mut root, 0.0, 0.0, 1920.0, 1080.0);
    let warm_time = start.elapsed();
    println!("Warm Layout (10k items): {:?}", warm_time);

    // 4. Invalidation (Dirty)
    root.mark_dirty();
    let start = Instant::now();
    compute_layout(&mut root, 0.0, 0.0, 1920.0, 1080.0);
    let dirty_time = start.elapsed();
    println!("Dirty Layout (10k items): {:?}", dirty_time);

    // Assert improvement
    if warm_time.as_micros() < 50 {
        println!("✅ Layout Cache working (Warm < 50us)");
    } else {
        println!("❌ Layout Cache ineffective (Warm {:?})", warm_time);
    }
}
