//! Widget state tracking for dirty detection and render caching.
//!
//! This module provides dirty tracking to skip rendering unchanged widgets.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use crate::rich_text::RichText;

/// Cached render state for a widget.
#[derive(Clone)]
pub struct WidgetRenderCache {
    /// Hash of widget content for change detection
    pub content_hash: u64,
    /// Cached parsed rich text (for labels, buttons, etc.)
    pub parsed_rich_text: Option<RichText>,
}

/// State tracker for widgets with dirty detection.
pub struct WidgetStateTracker {
    /// Cache by widget ID or hash
    cache: HashMap<u64, WidgetRenderCache>,
}

impl WidgetStateTracker {
    /// Creates a new empty state tracker.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
    
    /// Computes a hash for any hashable value.
    pub fn compute_hash<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Checks if a widget's text content has changed and returns
    /// cached RichText if available.
    /// Returns (is_dirty, Option<cached_rich_text>)
    pub fn check_text_cache(
        &mut self,
        widget_id: u64,
        text: &str,
    ) -> (bool, Option<RichText>) {
        let text_hash = Self::compute_hash(&text);
        
        if let Some(cached) = self.cache.get(&widget_id) {
            if cached.content_hash == text_hash {
                // Not dirty - return cached value
                return (false, cached.parsed_rich_text.clone());
            }
        }
        
        // Dirty - needs re-parsing
        (true, None)
    }

    /// Tries to retrieve cached rich text by content hash directly.
    pub fn get_by_hash(&mut self, hash: u64) -> Option<RichText> {
        if let Some(cached) = self.cache.get(&hash) {
             return cached.parsed_rich_text.clone();
        }
        None
    }

    /// Stores rich text by hash key.
    pub fn store_by_hash(&mut self, hash: u64, rich_text: RichText) {
        self.cache.insert(hash, WidgetRenderCache {
            content_hash: hash, // Self-referential for content addressing
            parsed_rich_text: Some(rich_text),
        });
    }
    
    /// Stores parsed rich text in cache.
    pub fn store_text_cache(
        &mut self,
        widget_id: u64,
        text: &str,
        rich_text: RichText,
    ) {
        let text_hash = Self::compute_hash(&text);
        self.cache.insert(widget_id, WidgetRenderCache {
            content_hash: text_hash,
            parsed_rich_text: Some(rich_text),
        });
    }
    
    /// Clears the entire cache (e.g., on resize).
    pub fn clear(&mut self) {
        self.cache.clear();
    }
    
    /// Returns the number of cached entries.
    pub fn len(&self) -> usize {
        self.cache.len()
    }
    
    /// Returns true if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Default for WidgetStateTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_hit() {
        let mut tracker = WidgetStateTracker::new();
        let text = "Hello World";
        let widget_id = 1;
        
        // First access - dirty
        let (is_dirty, cached) = tracker.check_text_cache(widget_id, text);
        assert!(is_dirty);
        assert!(cached.is_none());
        
        // Store cache
        use crate::rich_text::{RichText, TextStyle};
        let rich_text = RichText::parse(text, TextStyle::default());
        tracker.store_text_cache(widget_id, text, rich_text);
        
        // Second access - not dirty
        let (is_dirty, cached) = tracker.check_text_cache(widget_id, text);
        assert!(!is_dirty);
        assert!(cached.is_some());
    }
    
    #[test]
    fn test_cache_invalidation() {
        let mut tracker = WidgetStateTracker::new();
        let widget_id = 1;
        
        // Store with one text
        use crate::rich_text::{RichText, TextStyle};
        let text1 = "Hello";
        let rich_text = RichText::parse(text1, TextStyle::default());
        tracker.store_text_cache(widget_id, text1, rich_text);
        
        // Check with different text - should be dirty
        let text2 = "World";
        let (is_dirty, _) = tracker.check_text_cache(widget_id, text2);
        assert!(is_dirty);
    }
}
