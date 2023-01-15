use crate::{
    effects::lerp,
    player::{Player, SystemOrder},
    AppState,
};
use bevy::{input::mouse::MouseWheel, prelude::*};
use iyes_loopless::prelude::*;

const CAM_Z: f32 = 100.; // Generally the highest depth and sprites past it will not be rendered

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::GameLoading, load_camera).add_system_set(
            ConditionSet::new()
                .run_in_state(AppState::Running)
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
                    scale: 0.3,
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

    let lerped_pos = lerp(cam_pos.translation.truncate(), player_pos.translation.truncate(), 0.15);
    cam_pos.translation = Vec3::new(lerped_pos.x, lerped_pos.y, CAM_Z);
}

fn zoom_camera(
    mut camera_query: Query<(&mut Transform, &Camera2d, &mut CamScrollLock)>,
    mut scroll_wheel: EventReader<MouseWheel>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let (mut cam, _, mut cam_lock) = camera_query.single_mut();

    if keyboard_input.just_released(KeyCode::L) {
        cam_lock.0 = !cam_lock.0;
        println!("Camlock set to {}", cam_lock.0)
    }
    if cam_lock.0 {
        return;
    }

    let zoom_scroll_speed = 0.15;
    for direction in scroll_wheel.iter() {
        cam.scale =
            (cam.scale + zoom_scroll_speed * direction.y).clamp(Vec3::new(0.2, 0.2, 0.2), Vec3::new(6.0, 6.0, 6.0));
    }
}
