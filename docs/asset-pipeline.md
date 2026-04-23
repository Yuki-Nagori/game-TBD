# 资产管线规范

本文档定义 Mod 资产包的标准格式与校验规则。

## 清单格式

每个 Mod 必须包含一个 `manifest.toml` 文件：

```toml
name = "my-mod"
version = "1.0.0"

[[assets]]
path = "assets/textures/sword.png"
type = "texture"

[[assets]]
path = "assets/models/hero.gltf"
type = "model"
```

## 字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 是 | Mod 名称，唯一标识 |
| `version` | string | 是 | SemVer 版本号 |
| `assets` | array | 是 | 资产列表 |
| `assets[].path` | string | 是 | 资源路径，必须以 `assets/` 开头 |
| `assets[].type` | string | 是 | 资源类型标记 |

## 支持的资源类型

| 扩展名 | 类型 | 说明 |
|--------|------|------|
| `.png` | texture | 纹理 |
| `.jpg` | texture | 纹理 |
| `.webp` | texture | 纹理 |
| `.gltf` | model | 3D 模型 |
| `.glb` | model | 3D 模型（二进制） |

## 验证规则

1. **必填字段**：`name`、`version`、`assets` 不能为空
2. **路径格式**：不允许 `..`，必须以 `assets/` 开头
3. **扩展名白名单**：仅限 `png`、`jpg`、`webp`、`gltf`、`glb`
4. **路径唯一性**：同一清单内路径不能重复
5. **文件存在性**：引用的文件必须存在
6. **版本格式**：必须包含至少一个 `.`，建议遵循 SemVer

## 目录结构约定

```
my-mod/
├── manifest.toml
├── assets/
│   ├── textures/
│   │   └── sword.png
│   └── models/
│       └── hero.gltf
```

## 版本兼容性

使用主版本号判断兼容性：
- `1.0.0` 与 `1.2.3` 兼容
- `1.0.0` 与 `2.0.0` 不兼容

## 加载方式

```rust
use ming_rpg::asset_manager::AssetManager;

let mut manager = AssetManager::new();
let manifest = manager.load_manifest("mods/my-mod/manifest.toml")?;
```
