//! 场景插件
//!
//! 职责：3D 场景初始化、方块建筑系统、场景切换
//! Lua 相关功能已迁移到 lua_command_plugin

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::collections::HashMap;

use crate::components::Player;
use crate::constants::*;
use crate::lua_api::LuaRuntime;
use crate::resources::EntityRegistry;

/// 场景颜色配置（从 Lua 读取）
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SceneColorsConfig {
    pub wall: ColorRgb,
    pub roof: ColorRgb,
    pub tree: ColorRgb,
    pub ground: ColorRgb,
}

impl Default for SceneColorsConfig {
    fn default() -> Self {
        Self {
            wall: ColorRgb { r: 0.86, g: 0.24, b: 0.18 },
            roof: ColorRgb { r: 0.9, g: 0.8, b: 0.1 },
            tree: ColorRgb { r: 0.2, g: 0.6, b: 0.2 },
            ground: ColorRgb { r: 0.3, g: 0.3, b: 0.3 },
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ColorRgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl From<ColorRgb> for Color {
    fn from(c: ColorRgb) -> Self {
        Color::srgb(c.r, c.g, c.b)
    }
}

/// 场景对象配置
#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct SceneObjectConfig {
    pub r#type: String,
    pub x: f32,
    pub z: f32,
    pub color: Option<String>,
}

/// 场景连接配置
#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct SceneConnectionConfig {
    pub to: String,
    pub x: f32,
    pub z: f32,
    pub name: String,
}

/// 单个场景配置
#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct SceneConfig {
    pub name: String,
    pub description: String,
    pub spawn_point: HashMap<String, f32>,
    pub ground_size: f32,
    pub objects: Vec<SceneObjectConfig>,
    pub connections: Vec<SceneConnectionConfig>,
}

/// 场景总配置
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ScenesConfig {
    pub current: String,
    pub scenes: HashMap<String, SceneConfig>,
}

impl Default for ScenesConfig {
    fn default() -> Self {
        let mut scenes = HashMap::new();
        scenes.insert(
            "suburb".to_string(),
            SceneConfig {
                name: "城郊".to_string(),
                description: "北京城外，一片宁静的土地".to_string(),
                spawn_point: {
                    let mut p = HashMap::new();
                    p.insert("x".to_string(), 0.0);
                    p.insert("y".to_string(), 1.0);
                    p.insert("z".to_string(), 0.0);
                    p
                },
                ground_size: 50.0,
                objects: vec![],
                connections: vec![],
            },
        );
        Self {
            current: "suburb".to_string(),
            scenes,
        }
    }
}

/// 当前场景状态
#[derive(Resource)]
pub struct CurrentScene {
    pub scene_id: String,
    pub config: SceneConfig,
}

impl Default for CurrentScene {
    fn default() -> Self {
        Self {
            scene_id: "suburb".to_string(),
            config: SceneConfig {
                name: "城郊".to_string(),
                description: "北京城外，一片宁静的土地".to_string(),
                spawn_point: {
                    let mut p = HashMap::new();
                    p.insert("x".to_string(), 0.0);
                    p.insert("y".to_string(), 1.0);
                    p.insert("z".to_string(), 0.0);
                    p
                },
                ground_size: 50.0,
                objects: vec![],
                connections: vec![],
            },
        }
    }
}

/// 运行时颜色配置资源
#[derive(Resource, Default)]
pub struct SceneColorRes {
    pub colors: SceneColorsConfig,
}

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityRegistry>()
            .init_resource::<CurrentScene>()
            .init_resource::<SceneColorRes>()
            .add_systems(Startup, (load_scene_config, spawn_scene).chain())
            .add_systems(Update, check_scene_switch_system);
    }
}

/// 加载场景配置
fn load_scene_config(
    lua: Res<LuaRuntime>,
    mut current_scene: ResMut<CurrentScene>,
    mut scene_colors: ResMut<SceneColorRes>,
) {
    let scenes_config: ScenesConfig = lua.get_config("SCENE_CONFIG").unwrap_or_else(|| {
        info!("使用默认场景配置");
        ScenesConfig::default()
    });

    // 加载颜色配置
    let colors_config: SceneColorsConfig = lua.get_config("SCENE_COLORS").unwrap_or_else(|| {
        info!("使用默认场景颜色配置");
        SceneColorsConfig::default()
    });
    scene_colors.colors = colors_config;

    info!("场景配置加载完成，当前场景: {}", scenes_config.current);

    // 设置当前场景
    if let Some(config) = scenes_config.scenes.get(&scenes_config.current) {
        current_scene.scene_id = scenes_config.current.clone();
        current_scene.config = config.clone();
        info!("当前场景: {} - {}", config.name, config.description);
    } else {
        warn!("场景 {} 未找到，使用默认", scenes_config.current);
    }
}

/// 生成场景：地面、光照、场景对象
fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    current_scene: Res<CurrentScene>,
    scene_colors: Res<SceneColorRes>,
) {
    info!("初始化 3D 场景");

    // 光照
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
        ..default()
    });

    // 地面（带碰撞）：向下偏移使碰撞体顶面与视觉平面对齐
    let ground_size = current_scene.config.ground_size;
    let ground_color: Color = scene_colors.colors.ground.clone().into();
    // Bevy 0.14: Plane3d::new 需要法线参数
    let ground_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(ground_size)));
    commands.spawn((
        PbrBundle {
            mesh: ground_mesh,
            material: materials.add(ground_color),
            transform: Transform::from_xyz(0.0, -0.1, 0.0),
            ..default()
        },
        Collider::cuboid(ground_size / 2.0, 0.1, ground_size / 2.0),
    ));

    // 从场景配置生成对象
    let wall_color: Color = scene_colors.colors.wall.clone().into();
    let roof_color: Color = scene_colors.colors.roof.clone().into();
    let tree_color: Color = scene_colors.colors.tree.clone().into();

    for obj in &current_scene.config.objects {
        match obj.r#type.as_str() {
            "building" => {
                let color = match obj.color.as_deref() {
                    Some("wall") => wall_color,
                    Some("roof") => roof_color,
                    _ => wall_color,
                };
                let mesh = meshes.add(Cuboid::new(WALL_SIZE, WALL_SIZE, WALL_SIZE));
                commands.spawn((
                    PbrBundle {
                        mesh,
                        material: materials.add(color),
                        transform: Transform::from_translation(Vec3::new(
                            obj.x,
                            WALL_SIZE / 2.0,
                            obj.z,
                        )),
                        ..default()
                    },
                    Collider::cuboid(WALL_SIZE / 2.0, WALL_SIZE / 2.0, WALL_SIZE / 2.0),
                ));
            }
            "tree" => {
                let mesh = meshes.add(Cuboid::new(TREE_SIZE, TREE_SIZE * 2.0, TREE_SIZE));
                commands.spawn((
                    PbrBundle {
                        mesh,
                        material: materials.add(tree_color),
                        transform: Transform::from_translation(Vec3::new(obj.x, TREE_SIZE, obj.z)),
                        ..default()
                    },
                    Collider::cuboid(TREE_SIZE / 2.0, TREE_SIZE, TREE_SIZE / 2.0),
                ));
            }
            _ => {
                warn!("未知的场景对象类型: {}", obj.r#type);
            }
        }
    }

    // 默认建筑（当配置为空时作为后备）
    if current_scene.config.objects.is_empty() {
        spawn_building_blocks(
            &mut commands,
            &mut meshes,
            &mut materials,
            &scene_colors.colors,
        );
    }

    info!("场景创建完成");
}

/// 场景切换检测系统
///
/// 检测玩家是否接近场景切换点
fn check_scene_switch_system(
    player_query: Query<&Transform, With<Player>>,
    current_scene: Res<CurrentScene>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation;

    // 检查是否接近场景切换点
    for connection in &current_scene.config.connections {
        let distance = Vec2::new(player_pos.x - connection.x, player_pos.z - connection.z).length();

        // TODO: 显示 UI 提示，允许玩家按键切换场景
        // 避免每帧记录日志造成性能问题
        let _ = distance;
    }
}

/// 生成方块建筑占位
fn spawn_building_blocks(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    colors: &SceneColorsConfig,
) {
    let wall_color: Color = colors.wall.clone().into();
    let roof_color: Color = colors.roof.clone().into();
    let tree_color: Color = colors.tree.clone().into();

    let wall_mesh = meshes.add(Cuboid::new(WALL_SIZE, WALL_SIZE, WALL_SIZE));
    let roof_mesh = meshes.add(Cuboid::new(ROOF_SIZE, ROOF_SIZE / 2.0, ROOF_SIZE));
    let tree_mesh = meshes.add(Cuboid::new(TREE_SIZE, TREE_SIZE * 2.0, TREE_SIZE));

    // 墙面
    let wall_positions = [
        Vec3::new(8.0, WALL_SIZE / 2.0, 0.0),
        Vec3::new(-8.0, WALL_SIZE / 2.0, 0.0),
        Vec3::new(0.0, WALL_SIZE / 2.0, 8.0),
        Vec3::new(0.0, WALL_SIZE / 2.0, -8.0),
    ];
    for pos in wall_positions {
        commands.spawn((
            PbrBundle {
                mesh: wall_mesh.clone(),
                material: materials.add(wall_color),
                transform: Transform::from_translation(pos),
                ..default()
            },
            Collider::cuboid(WALL_SIZE / 2.0, WALL_SIZE / 2.0, WALL_SIZE / 2.0),
        ));
    }

    // 屋顶
    commands.spawn((
        PbrBundle {
            mesh: roof_mesh,
            material: materials.add(roof_color),
            transform: Transform::from_translation(Vec3::new(
                -8.0,
                WALL_SIZE / 2.0 + ROOF_SIZE / 4.0,
                -8.0,
            )),
            ..default()
        },
        Collider::cuboid(ROOF_SIZE / 2.0, ROOF_SIZE / 4.0, ROOF_SIZE / 2.0),
    ));

    // 树木
    let tree_positions = [
        Vec3::new(5.0, TREE_SIZE, 5.0),
        Vec3::new(-5.0, TREE_SIZE, -5.0),
        Vec3::new(5.0, TREE_SIZE, -5.0),
    ];
    for pos in tree_positions {
        commands.spawn((
            PbrBundle {
                mesh: tree_mesh.clone(),
                material: materials.add(tree_color),
                transform: Transform::from_translation(pos),
                ..default()
            },
            Collider::cuboid(TREE_SIZE / 2.0, TREE_SIZE, TREE_SIZE / 2.0),
        ));
    }
}
