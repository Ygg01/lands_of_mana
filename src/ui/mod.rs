use bevy_pixel_camera::PixelProjection;
use kayak_ui::bevy::BevyContext;
use leafwing_input_manager::prelude::*;

use crate::prelude::*;

mod camera;
mod gui;
pub struct InputPlugin {}

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(bevy::input::system::exit_on_esc_system)
            .add_system(bevy::window::exit_on_window_close_system)
            .add_plugin(InputManagerPlugin::<InputActions>::default())
            .add_enter_system(config::EngineState::InGame, setup)
            .add_system_set(
                ConditionSet::new()
                    .label_and_after(config::UpdateStageLabel::Input)
                    .run_in_state(config::EngineState::InGame)
                    .with_system(input_to_game_actions)
                    .with_system(interact)
                    .into(),
            )
            .add_plugin(camera::CameraPlugin {})
            .add_plugin(gui::GuiPlugin {});
    }
}

fn setup(mut commands: Commands, world_query: Query<Entity, With<game::GameWorld>>) {
    let world_entity = world_query.single();
    let mut input_map = InputMap::new([
        // pause / resume
        (InputActions::TogglePause, KeyCode::Space),
        (InputActions::CameraMoveNorth, KeyCode::W),
        (InputActions::CameraMoveSouth, KeyCode::S),
        (InputActions::CameraMoveWest, KeyCode::A),
        (InputActions::CameraMoveEast, KeyCode::D),
        (InputActions::CameraZoomIn, KeyCode::Z),
        (InputActions::CameraZoomOut, KeyCode::X),
    ]);
    input_map.insert(InputActions::Interact, MouseButton::Left);
    commands
        .entity(world_entity)
        .insert_bundle(InputManagerBundle::<InputActions> {
            action_state: ActionState::default(),
            input_map,
        });
}

fn input_to_game_actions(
    game_state: Res<CurrentState<game::InGameState>>,
    input_action_query: Query<&ActionState<InputActions>>,
    mut world_action_query: Query<&mut ActionState<game::actions::WorldActions>>,
) {
    let input_action_state = input_action_query.single();
    let mut world_action_state = world_action_query.single_mut();

    if input_action_state.just_pressed(InputActions::Pause)
        || (input_action_state.just_pressed(InputActions::TogglePause)
            && game_state.0 == game::InGameState::Running)
    {
        world_action_state.press(game::actions::WorldActions::Pause)
    }

    if input_action_state.just_pressed(InputActions::Resume)
        || (input_action_state.just_pressed(InputActions::TogglePause)
            && game_state.0 == game::InGameState::Paused)
    {
        world_action_state.press(game::actions::WorldActions::Resume)
    }
}

fn interact(
    mut commands: Commands,
    windows: Res<Windows>,
    kayak_context_option: Option<Res<BevyContext>>,
    camera_transform_query: Query<(&Camera, &Transform), With<PixelProjection>>,
    map_query: Query<&game::map::Map>,
    input_action_query: Query<&ActionState<InputActions>>,
    selectable_query: Query<(Entity, &game::map::Position), With<Selectable>>,
) {
    let input_action_state = input_action_query.single();
    let ui_contains_cursor = match kayak_context_option {
        Some(kayak_context) => kayak_context.contains_cursor(),
        None => false,
    };

    if input_action_state.just_pressed(InputActions::Interact) && !ui_contains_cursor {
        let window = windows.get_primary().unwrap();

        let (camera, camera_transform) = camera_transform_query.single();
        if let Some(pixel_position) =
            camera::camera_position_to_pixel_position(window, camera, camera_transform)
        {
            let map = map_query.single();
            let cursor_position = map.pixel_position_to_position(pixel_position);

            for (entity, position) in selectable_query.iter() {
                println!("{:?}", (position, &cursor_position));
                if &cursor_position == position {
                    commands.entity(entity).insert(Selected {});
                } else {
                    commands.entity(entity).remove::<Selected>();
                }
            }
        }
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum InputActions {
    Pause,
    Resume,
    TogglePause,

    CameraMoveNorth,
    CameraMoveSouth,
    CameraMoveWest,
    CameraMoveEast,
    CameraZoomIn,
    CameraZoomOut,

    // Generic left click interact
    Interact,
}

#[derive(Component, Debug, Clone, Default)]
pub struct Selectable {}

#[derive(Component, Debug, Clone, Default)]
#[component(storage = "SparseSet")]
pub struct Selected {}

#[derive(Component, Debug, Clone, Default)]
#[component(storage = "SparseSet")]
pub struct Selecting {}
