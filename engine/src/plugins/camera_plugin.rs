//! 相机插件
//!
//! 第三人称相机：跟随、鼠标控制、球面坐标
//!
//! 控制方案（陀螺仪模式）：
//! - 鼠标移动：直接控制相机旋转
//! - 鼠标锁定在窗口中心，隐藏光标
//! - 按 Alt 键释放/锁定鼠标

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use crate::components::{Player, ThirdPersonCamera};
use crate::constants::{
    CAMERA_DISTANCE_DEFAULT, CAMERA_DISTANCE_MAX, CAMERA_DISTANCE_MIN, CAMERA_MOUSE_SENSITIVITY,
    CAMERA_PITCH_MAX, CAMERA_PITCH_MIN, CAMERA_SMOOTH_FACTOR_DEFAULT, CAMERA_ZOOM_SPEED,
};
use crate::lua_api::LuaRuntime;
use crate::resources::CameraState;

/// 相机配置（从 Lua 读取）
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CameraConfig {
    #[serde(rename = "distance")]
    pub distance: f32,
    #[serde(rename = "smooth_factor")]
    pub smooth_factor: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            distance: CAMERA_DISTANCE_DEFAULT,
            smooth_factor: CAMERA_SMOOTH_FACTOR_DEFAULT,
        }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraState>()
            .add_systems(Startup, (spawn_camera, setup_mouse))
            .add_systems(
                Update,
                (
                    toggle_mouse_lock_system,
                    camera_mouse_follow_system,
                    camera_zoom_system,
                    camera_follow_system,
                    check_mouse_lock_status_system,
                ),
            );
    }
}

/// 生成相机实体
fn spawn_camera(
    mut commands: Commands,
    mut camera_state: ResMut<CameraState>,
    lua: Res<LuaRuntime>,
) {
    // 尝试从 Lua 读取相机配置
    let config: CameraConfig = lua.get_config("CAMERA_CONFIG").unwrap_or_else(|| {
        info!("使用默认相机配置");
        CameraConfig::default()
    });

    info!("相机配置: {:?}", config);

    // 应用配置到相机状态
    camera_state.distance = config
        .distance
        .clamp(CAMERA_DISTANCE_MIN, CAMERA_DISTANCE_MAX);
    camera_state.smooth_factor = config.smooth_factor;

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 10.0, config.distance)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ThirdPersonCamera,
    ));

    info!("第三人称相机创建完成");
}

/// 设置鼠标：锁定并隐藏
///
/// 错误处理：
/// - 如果窗口不存在，记录错误
/// - 如果锁定失败（某些平台/窗口模式不支持），显示警告并继续运行
fn setup_mouse(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    let Ok(mut window) = window_query.get_single_mut() else {
        error!("无法获取主窗口，鼠标锁定失败");
        return;
    };

    // 尝试锁定鼠标
    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;

    // 检查是否成功锁定
    if window.cursor.grab_mode == CursorGrabMode::Locked {
        info!("鼠标已锁定（陀螺仪模式），按 Alt 键可释放鼠标");
    } else {
        warn!("鼠标锁定失败（可能是不支持的窗口模式），按 Alt 键重试");
    }
}

/// 鼠标状态检查系统
///
/// 定期检查鼠标锁定状态，确保游戏始终知道当前状态
/// 处理边界情况：窗口失去焦点后重新获得焦点时恢复锁定
fn check_mouse_lock_status_system(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_state: ResMut<CameraState>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };

    let is_actually_locked = window.cursor.grab_mode == CursorGrabMode::Locked;

    // 如果实际状态与记录状态不一致，更新记录
    if is_actually_locked != camera_state.mouse_locked {
        camera_state.mouse_locked = is_actually_locked;
        if is_actually_locked {
            info!("鼠标锁定状态：已锁定");
        } else {
            info!("鼠标锁定状态：已释放");
        }
    }
}

/// 切换鼠标锁定状态系统
///
/// 按 Alt 键在锁定/释放鼠标之间切换
fn toggle_mouse_lock_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut camera_state: ResMut<CameraState>,
) {
    let Ok(mut window) = window_query.get_single_mut() else {
        return;
    };

    // Alt 键按下时切换鼠标锁定状态
    if keyboard.just_pressed(KeyCode::AltLeft) || keyboard.just_pressed(KeyCode::AltRight) {
        let is_locked = window.cursor.grab_mode == CursorGrabMode::Locked;
        if is_locked {
            // 释放鼠标
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
            camera_state.mouse_locked = false;
            info!("鼠标已释放（可交互模式）");
        } else {
            // 锁定鼠标
            window.cursor.grab_mode = CursorGrabMode::Locked;
            window.cursor.visible = false;
            camera_state.mouse_locked = true;
            info!("鼠标已锁定（陀螺仪模式）");
        }
    }
}

/// 相机鼠标跟随系统（陀螺仪模式）
///
/// 鼠标移动直接控制相机旋转，提供流畅的视角控制
/// 仅在鼠标锁定时生效
fn camera_mouse_follow_system(
    mut camera_state: ResMut<CameraState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut motion_events: EventReader<MouseMotion>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };

    // 仅在鼠标锁定时处理相机旋转
    if window.cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }

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

/// 相机滚轮缩放系统
///
/// 使用鼠标滚轮调整相机距离
fn camera_zoom_system(
    mut camera_state: ResMut<CameraState>,
    mut scroll_events: EventReader<MouseWheel>,
) {
    let mut scroll_delta = 0.0;
    for event in scroll_events.read() {
        scroll_delta += event.y;
    }

    if scroll_delta != 0.0 {
        // 调整相机距离（滚轮向上减小距离，滚轮向下增加距离）
        camera_state.distance -= scroll_delta * CAMERA_ZOOM_SPEED;
        // 限制在最小/最大范围内
        camera_state.distance = camera_state
            .distance
            .clamp(CAMERA_DISTANCE_MIN, CAMERA_DISTANCE_MAX);
    }
}

/// 相机跟随系统
///
/// 使相机基于球面坐标跟随玩家
/// 性能优化：使用栈上计算，避免堆分配
fn camera_follow_system(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<ThirdPersonCamera>, Without<Player>)>,
    camera_state: Res<CameraState>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    // 球面坐标计算（使用动态距离）
    let radius = camera_state.distance;
    let yaw = camera_state.yaw;
    let pitch = camera_state.pitch.clamp(CAMERA_PITCH_MIN, CAMERA_PITCH_MAX);

    // 性能优化：球面坐标转笛卡尔坐标（栈上计算）
    let cos_pitch = pitch.cos();
    let sin_pitch = pitch.sin();
    let cos_yaw = yaw.cos();
    let sin_yaw = yaw.sin();

    let x = radius * cos_pitch * sin_yaw;
    let y = radius * sin_pitch;
    let z = radius * cos_pitch * cos_yaw;

    let player_position = player_transform.translation;
    let target_position = Vec3::new(
        player_position.x + x,
        player_position.y + y,
        player_position.z + z,
    );

    // 平滑移动相机（帧率无关的指数衰减公式）
    let smooth_factor = 1.0 - (1.0 - camera_state.smooth_factor).powf(time.delta_seconds() * 60.0);
    camera_transform.translation = camera_transform
        .translation
        .lerp(target_position, smooth_factor);

    // 相机始终看向玩家
    camera_transform.look_at(player_position, Vec3::Y);
}
