# Template 系统设计

> 剧本创作工具链 —— 让写故事像填空一样简单

**状态**：📝 规划中（Phase 4 后开发）

---

## 目标

为剧本创作者提供：
1. **开箱即用的模板** — 复制即可开始写
2. **自动化验证** — 提交前检查错误
3. **一键打包发布** — 生成标准 .mod 文件

---

## 架构分层

```
┌─────────────────────────────────────┐
│         剧本创作者（用户）            │
│    使用 Template 仓库创作剧本         │
└─────────────┬───────────────────────┘
              │ git push
┌─────────────▼───────────────────────┐
│      Template 仓库（GitHub）          │
│  - 标准目录结构                       │
│  - 示例代码                          │
│  - CI 验证流程                        │
└─────────────┬───────────────────────┘
              │ 打 Tag
┌─────────────▼───────────────────────┐
│      GitHub Actions 自动化            │
│  - 验证 mod.json 格式                 │
│  - 检查 Lua 语法                      │
│  - 打包 .mod 文件                     │
│  - 发布到 Releases                    │
└─────────────┬───────────────────────┘
              │ 下载
┌─────────────▼───────────────────────┐
│         玩家                          │
│    放入 mods/ 目录，启动游戏           │
└─────────────────────────────────────┘
```

---

## Template 仓库结构

```
ming-rpg-template/           # 模板仓库（单独 GitHub 仓库）
├── README.md                # 使用说明
├── LICENSE                  # 剧本授权协议（建议 MIT/CC-BY）
├── mod.json                 # 模板元数据（需修改）
├── init.lua                 # 入口脚本模板
├── .github/
│   └── workflows/
│       ├── validate.yml     # PR 时验证
│       └── release.yml      # 打 Tag 时打包
├── scripts/
│   ├── 主线/
│   │   └── 示例事件.lua      # 事件脚本示例
│   └── 支线/
│       └── 示例支线.lua
├── entities/
│   ├── 角色/
│   │   └── 示例角色.json     # 角色配置示例
│   └── 物品/
│       └── 示例物品.json
├── scenes/
│   └── 示例场景/
│       └── scene.json       # 场景配置示例
├── assets/
│   ├── models/              # 3D 模型资源
│   ├── textures/            # 贴图资源
│   └── sounds/              # 音效资源
└── tools/
    ├── check-mod.sh         # 本地验证脚本
    └── create-scene.py      # 场景生成辅助工具
```

---

## 使用流程

### 创作者：从模板到发布

```bash
# 1. 在 GitHub 点击 "Use this template" 创建新仓库
#    命名为：我的剧本名

# 2. 克隆到本地
git clone https://github.com/你的用户名/我的剧本.git
cd 我的剧本

# 3. 修改元数据
vim mod.json
# {
#   "id": "my-story",
#   "name": "我的剧本名称",
#   "version": "0.1.0",
#   "author": "你的名字",
#   "description": "剧本简介..."
# }

# 4. 编写剧本
vim init.lua              # 入口脚本
vim scripts/主线/第一章.lua  # 事件脚本
vim entities/角色/主角.json  # 角色配置

# 5. 本地验证
./tools/check-mod.sh
# ✓ mod.json 格式正确
# ✓ Lua 语法检查通过
# ✓ 资源文件存在

# 6. 提交到 GitHub
git add .
git commit -m "初始剧本"
git push

# 7. 打 Tag 发版
git tag v0.1.0
git push origin v0.1.0

# 8. GitHub Actions 自动：
#    - 验证剧本
#    - 打包为 我的剧本-v0.1.0.mod
#    - 发布到 Releases 页面

# 9. 分享下载链接给玩家！
```

### 玩家：安装剧本

```bash
# 1. 从 Releases 下载 .mod 文件
# 2. 放入游戏目录
mv 我的剧本-v0.1.0.mod ~/game/mods/

# 3. 启动游戏，在剧本选择界面看到 "我的剧本名称"
# 4. 开始游戏！
```

---

## CI 自动化流程

### validate.yml（PR 时触发）

```yaml
name: Validate Mod

on: [pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Validate mod.json
        run: |
          python3 tools/validate-mod-json.py mod.json
      
      - name: Check Lua syntax
        run: |
          sudo apt-get install -y lua5.4
          find scripts -name "*.lua" -exec lua5.4 -c {} \;
      
      - name: Check resources exist
        run: |
          python3 tools/check-resources.py
```

### release.yml（打 Tag 时触发）

```yaml
name: Release Mod

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Get version from tag
        id: get_version
        run: echo "VERSION=${GITHUB_REF#refs/tags/}" >> $GITHUB_OUTPUT
      
      - name: Package mod
        run: |
          MOD_NAME=$(python3 -c "import json; print(json.load(open('mod.json'))['id'])")
          zip -r "${MOD_NAME}-${{ steps.get_version.outputs.VERSION }}.mod" \
            mod.json init.lua scripts/ entities/ scenes/ assets/
      
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: "*.mod"
```

---

## 本地工具

### check-mod.sh

```bash
#!/bin/bash
# 本地验证脚本

echo "🔍 验证 mod.json..."
python3 tools/validate-mod-json.py mod.json || exit 1

echo "🔍 检查 Lua 语法..."
find scripts -name "*.lua" -exec lua5.4 -c {} \; || exit 1

echo "🔍 检查资源引用..."
python3 tools/check-resources.py || exit 1

echo "✅ 所有检查通过！"
```

### validate-mod-json.py

```python
#!/usr/bin/env python3
"""验证 mod.json 格式"""

import json
import sys

REQUIRED_FIELDS = ['id', 'name', 'version', 'author', 'description', 'entry']

def validate(path):
    with open(path) as f:
        data = json.load(f)
    
    for field in REQUIRED_FIELDS:
        if field not in data:
            print(f"❌ 缺少必需字段: {field}")
            return False
    
    print("✅ mod.json 格式正确")
    return True

if __name__ == '__main__':
    sys.exit(0 if validate(sys.argv[1]) else 1)
```

---

## 开发时机

| 阶段 | 任务 | 优先级 |
|:---|:---|:---:|
| **Phase 1-3** | 专注引擎开发 | - |
| **Phase 4** | 明朝修仙 MVP 完成 | - |
| **Phase 4 后** | 提取 Template 仓库 | 🔥 |
| **Phase 4 后** | 编写模板文档和示例 | 🔥 |
| **EA 发布前** | 完善 CI 自动化 | 📝 |
| **EA 发布后** | 根据创作者反馈迭代 | 📝 |

---

## 相关文档

- [Lua API 设计](./lua-api.md) — 剧本可用的接口
- [Mod 系统架构](./mod-system.md) — 加载与管理机制
- [剧本设计指南](./mod-system.md#剧本设计指南) — 创作最佳实践

---

*「好的工具让创作者专注于故事本身」*
