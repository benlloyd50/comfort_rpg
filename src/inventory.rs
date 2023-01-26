use bevy::prelude::*;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(AppState::GameLoading, create_inventory_ui)
            .add_system(
                pickup_item.run_in_state(AppState::Running)
                .label(SystemOrder::Input)
                .before(SystemOrder::Logic)
            );
    }
}

/// When the player presses the pickup key it will attempt to pickup the item under the player or
/// in the direction they face, priority is given to underneath self
fn pickup_item() {
    todo!()
}

fn create_inventory_ui() {
    todo!()
}
