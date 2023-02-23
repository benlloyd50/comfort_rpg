use crate::{
    player::{Player, SystemOrder},
    GameState,
};
use bevy::{input::mouse::MouseWheel, prelude::*};
use iyes_loopless::prelude::*;

const CAM_Z: f32 = 100.; // Generally the highest depth and sprites past it will not be rendered

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::GameLoading, load_camera).add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Running)
                .label("graphicDelay")
                .after(SystemOrder::Graphic)
                .with_system(zoom_camera)
                .with_system(camera_follow_player)
                .into(),
        );
    }
}

#[derive(Component)]
struct CamScrollLock(bool);

fn load_camera(mut commands: Commands) {
    let _camera_entity = commands
        .spawn((
            Camera2dBundle {
                transform: Transform::from_xyz(0.0, 0.0, CAM_Z),
                projection: OrthographicProjection {
                    scale: 0.5,
                    ..default()
                },
                ..default()
            },
            CamScrollLock(false),
        ))
        .id();
    println!("Camera loaded succesfully");
}

fn camera_follow_player(
    mut camera: Query<&mut Transform, With<Camera2d>>,
    player_q: Query<&Transform, (With<Player>, Without<Camera2d>)>,
) {
    let mut cam_pos = camera.single_mut();
    let player_pos = player_q.single();

    cam_pos.translation = player_pos.translation;
    // Bound the position by the map so we dont see what's past it
    // let x_offset = cam_pos.scale.x * 39.5 * TILE_PIXELS_X;
    // let y_offset = cam_pos.scale.x * 39.5 * TILE_PIXELS_Y;
    // let x_bound = MAP_SIZE_X as f32 * TILE_PIXELS_X - x_offset - 8.0;
    // let y_bound = MAP_SIZE_Y as f32 * TILE_PIXELS_Y - y_offset - 8.0;
    // let new_x = if x_offset < x_bound {
    //     lerped_pos.x.clamp(x_offset, x_bound)
    // } else {
    //     lerped_pos.x.clamp(x_bound, x_offset)
    // };
    // let new_y = if y_offset < y_bound {
    //     lerped_pos.y.clamp(y_offset, y_bound)
    // } else {
    //     lerped_pos.y.clamp(y_bound, y_offset)
    // };
}

fn zoom_camera(
    mut camera_query: Query<(&mut OrthographicProjection, &mut CamScrollLock)>,
    mut scroll_wheel: EventReader<MouseWheel>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let (mut ortho, mut cam_lock) = camera_query.single_mut();

    if keyboard_input.just_released(KeyCode::L) {
        cam_lock.0 = !cam_lock.0;
        println!("Camlock set to {}", cam_lock.0)
    }
    if cam_lock.0 {
        return;
    }

    let zoom_scroll_speed = 0.1;
    for direction in scroll_wheel.iter() {
        ortho.scale = ((ortho.scale + zoom_scroll_speed * direction.y).clamp(0.1, 1.0) * 10.0).round() / 10.0;
        println!("{}", ortho.scale);
    }
}
