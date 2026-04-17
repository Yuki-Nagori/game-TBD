//! 相机插件
//!
//! 第三人称相机：跟随、鼠标控制、球面坐标
//!
//! 控制方案（陀螺仪模式）：
//! - 鼠标移动：直接控制相机旋转
//! - 鼠标锁定在窗口中心，隐藏光标

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use crate::components::{Player, ThirdPersonCamera};
use crate::constants::{
    CAMERA_DISTANCE, CAMERA_MOUSE_SENSITIVITY, CAMERA_PITCH_MAX, CAMERA_PITCH_MIN,
    CAMERA_SMOOTH_FACTOR,
};
use crate::resources::CameraState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraState>()
            .add_systems(Startup, (spawn_camera, setup_mouse))
            .add_systems(Update, (camera_mouse_follow_system, camera_follow_system));
    }
}

/// 生成相机实体
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ThirdPersonCamera,
    ));

    info!("第三人称相机创建完成");
}

/// 设置鼠标：锁定并隐藏
fn setup_mouse(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    let Ok(mut window) = window_query.get_single_mut() else {
        return;
    };

    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;

    info!("鼠标已锁定（陀螺仪模式）");
}

/// 相机鼠标跟随系统（陀螺仪模式）
///
/// 鼠标移动直接控制相机旋转，提供流畅的视角控制
fn camera_mouse_follow_system(
    mut camera_state: ResMut<CameraState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut motion_events: EventReader<MouseMotion>,
) {
    let Ok(_window) = window_query.get_single() else {
        return;
    };

    // 累积鼠标移动增量
    let mut delta = Vec2::ZERO;
    for motion in motion_events.read() {
        delta += motion.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    // 应用灵敏度
    camera_state.yaw -= delta.x * CAMERA_MOUSE_SENSITIVITY;
    camera_state.pitch -= delta.y * CAMERA_MOUSE_SENSITIVITY;

    // 限制俯仰角
    camera_state.pitch = camera_state.pitch.clamp(CAMERA_PITCH_MIN, CAMERA_PITCH_MAX);
}

/// 相机跟随系统
///
/// 使相机基于球面坐标跟随玩家
fn camera_follow_system(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<ThirdPersonCamera>, Without<Player>)>,
    camera_state: Res<CameraState>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    // 球面坐标计算
    let radius = CAMERA_DISTANCE;
    let yaw = camera_state.yaw;
    let pitch = camera_state.pitch.clamp(CAMERA_PITCH_MIN, CAMERA_PITCH_MAX);

    // 球面坐标转笛卡尔坐标
    let x = radius * pitch.cos() * yaw.sin();
    let y = radius * pitch.sin();
    let z = radius * pitch.cos() * yaw.cos();

    let player_position = player_transform.translation;
    let target_position = player_position + Vec3::new(x, y, z);

    // 平滑移动相机
    camera_transform.translation = camera_transform
        .translation
        .lerp(target_position, CAMERA_SMOOTH_FACTOR);

    // 相机始终看向玩家
    camera_transform.look_at(player_position, Vec3::Y);
}
