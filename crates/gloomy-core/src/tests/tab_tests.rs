use crate::widget::{Widget, TabItem, TabStyle, Orientation};
use crate::layout_engine::compute_layout;
use crate::ui::{hit_test, handle_interactions};
use crate::interaction::InteractionState;
use crate::Vec2;

#[test]
fn test_tab_layout_and_interaction() {
    let mut tab_widget = Widget::tab(
        "my_tab",
        vec![
            TabItem { title: "Tab 1".into(), content: Box::new(Widget::label("Content 1")) },
            TabItem { title: "Tab 2".into(), content: Box::new(Widget::label("Content 2")) },
        ],
        Orientation::Horizontal,
        TabStyle::default(),
    );
    
    // 1. Initial Bounds Setup (Root widget needs bounds set explicitly)
    if let Widget::Tab { bounds, .. } = &mut tab_widget {
         bounds.width = 800.0;
         bounds.height = 600.0;
    }

    // 2. Compute Layout
    // Parent 800x600
    compute_layout(&mut tab_widget, 0.0, 0.0, 800.0, 600.0);
    
    if let Widget::Tab { bounds, .. } = &tab_widget {
         assert_eq!(bounds.width, 800.0);
         assert_eq!(bounds.height, 600.0);
    } else {
         panic!("Not a tab widget");
    }
    
    // 3. Simulate Click on Tab 2 (Index 1)
    // Header height is 32.0. Width is 800.0. 2 Tabs. Each tab width 400.0.
    // Tab 0: 0..400. Tab 1: 400..800.
    // Click at 600, 16.
    
    let click_pos = Vec2::new(600.0, 16.0);
    let mut interaction = InteractionState::new();
    
    let hit = hit_test(&tab_widget, click_pos, Some(&interaction));
    assert!(hit.is_some(), "Should hit tab header");
    let hit = hit.unwrap();
    assert_eq!(hit.action, "my_tab:tab:1");
    
    // 3. Handle Interaction
    interaction.clicked_id = Some(hit.action);
    let changed = handle_interactions(&mut tab_widget, &interaction, Vec2::ZERO);
    assert!(changed, "Interaction should trigger change");
    
    // 4. Verify Selection
    if let Widget::Tab { selected, .. } = &tab_widget {
         assert_eq!(*selected, 1, "Tab 1 should be selected");
    } else {
         panic!("Not a tab widget");
    }
}
