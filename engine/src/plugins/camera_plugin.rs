//! 相机插件
//!
//! 第三人称相机：跟随、鼠标控制、球面坐标
//!
//! 控制方案：
//! - 鼠标移动：相机跟随（类似《原神》）
//! - 右键：技能释放（战斗时）

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{Player, ThirdPersonCamera};
use crate::constants::{
    CAMERA_DISTANCE, CAMERA_EDGE_THRESHOLD, CAMERA_MOUSE_FOLLOW_SPEED, CAMERA_PITCH_MAX,
    CAMERA_PITCH_MIN, CAMERA_SMOOTH_FACTOR,
};
use crate::resources::CameraState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraState>()
            .add_systems(Startup, spawn_camera)
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

/// 相机跟随鼠标系统（类似《原神》）
///
/// 鼠标在屏幕边缘时，相机缓慢旋转跟随
fn camera_mouse_follow_system(
    mut camera_state: ResMut<CameraState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };

    // 获取鼠标位置
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let window_size = Vec2::new(window.width(), window.height());
    let center = window_size / 2.0;
    let delta = cursor_pos - center;

    // 定义边缘区域（屏幕边缘区域触发旋转）
    let edge_threshold_x = window_size.x * CAMERA_EDGE_THRESHOLD;
    let edge_threshold_y = window_size.y * CAMERA_EDGE_THRESHOLD;

    // 水平旋转
    if delta.x.abs() > edge_threshold_x {
        let direction = delta.x.signum();
        camera_state.yaw -= direction * CAMERA_MOUSE_FOLLOW_SPEED * time.delta_seconds();
    }

    // 垂直旋转（限制范围）
    if delta.y.abs() > edge_threshold_y {
        let direction = delta.y.signum();
        camera_state.pitch += direction * CAMERA_MOUSE_FOLLOW_SPEED * time.delta_seconds();
        camera_state.pitch = camera_state.pitch.clamp(CAMERA_PITCH_MIN, CAMERA_PITCH_MAX);
    }
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
