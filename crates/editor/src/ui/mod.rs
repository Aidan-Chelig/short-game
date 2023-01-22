use bevy::prelude::*;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;

use self::{
    state::UiState,
    viewport::{auto_add_raycast_target, handle_pick_events, set_camera_viewport, set_gizmo_mode},
};

pub use self::viewport::MainCamera;

mod state;
mod viewport;

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiState::new())
            .add_plugin(DefaultInspectorConfigPlugin)
            .add_plugin(bevy_egui::EguiPlugin)
            .add_plugins(bevy_mod_picking::plugins::DefaultPickingPlugins)
            .add_system(set_camera_viewport)
            .add_system(set_gizmo_mode)
            .add_system(auto_add_raycast_target)
            .add_system(handle_pick_events)
            .add_system_to_stage(CoreStage::PreUpdate, show_ui_system.at_end());
    }
}

pub fn show_ui_system(world: &mut World) {
    let mut egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| ui_state.ui(world, &mut egui_context));
}

#[derive(Debug)]
pub enum Window {
    GameView,
    Hierarchy,
    Resources,
    Assets,
    Inspector,
}
