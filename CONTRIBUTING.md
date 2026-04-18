# 贡献指南

> 「代码是写给人看的，顺便给机器执行」—— Donald Knuth

## 开发流程

```
1. 创建功能分支
   git checkout -b feature/xxx

2. 开发并测试
   xmake format
   xmake check
   xmake run

3. 提交更改
   git commit -m "type: 描述"

4. 推送并创建PR
   git push origin feature/xxx
```

## 提交信息规范

```
<type>: <subject>

<body>

<footer>
```

### Type
- `feat`: 新功能
- `fix`: Bug修复
- `docs`: 文档
- `style`: 格式调整
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试
- `chore`: 工具/构建

### 示例
```
feat: 添加热重载系统

- 监听Lua文件变化
- 自动重载不重启游戏
- 添加F5手动重载快捷键

Refs: #42
```

## 代码规范

### Rust
- 遵循 `rustfmt` 格式
- 通过 `clippy` 检查（零警告）
- 文档注释使用 `///`

### Lua
- 遵循 `stylua` 格式
- 通过 `luacheck` 检查（零警告）
- 使用 `--` 单行注释，`--[[ ]]` 多行注释

## 测试要求

- 新功能必须包含测试
- 修复Bug必须包含回归测试
- 所有测试必须通过 `xmake check`

## 目录结构

```
engine/src/       # Rust引擎代码
game/             # Lua游戏逻辑
docs/             # 文档
tests/            # 测试
tools/            # 开发工具
```

## 问题反馈

发现Bug或有建议？请创建Issue并包含：
1. 问题描述
2. 复现步骤
3. 期望行为
4. 实际行为
5. 环境信息（OS, Rust版本等）

---
感谢你的贡献！🎹
