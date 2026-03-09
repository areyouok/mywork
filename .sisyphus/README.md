# Sisyphus 工作流元数据

此目录包含 Sisyphus AI 工作流系统的元数据文件。

## 目录结构

```
.sisyphus/
├── plans/                    # 📋 工作计划（重要 - 保留）
│   └── mywork-scheduler.md   # 项目完整需求文档
├── evidence/                 # 📸 工作证据（临时 - 可删除）
│   ├── final-qa/            # Final Wave 验证报告
│   └── task-*.txt/png       # 各任务证据和截图
├── notepads/                 # 📝 学习笔记（重要 - 保留）
│   └── mywork-scheduler/
│       └── learnings.md     # 技术决策和最佳实践
└── boulder.json              # 🔄 当前工作状态（可删除）
```

## 是否删除？

### ✅ 建议保留（重要文档）

- **`plans/mywork-scheduler.md`** - 完整需求文档，对维护和新成员有价值
- **`notepads/mywork-scheduler/learnings.md`** - 技术决策和最佳实践

### ⚠️ 可以删除（临时文件）

以下文件已被 `.gitignore` 排除，不会提交到 git：

- **`evidence/`** - 测试截图、日志等临时证据
- **`boulder.json`** - 当前工作会话状态
- **`notepads/`** - 如果不需要保留学习笔记

### 删除命令

如果确定要删除临时文件：

```bash
# 删除临时证据和工作状态（保留计划和笔记）
rm -rf .sisyphus/evidence/
rm -f .sisyphus/boulder.json

# 或者删除整个 .sisyphus 目录（不推荐）
# rm -rf .sisyphus/
```

## 当前状态

✅ **项目已完成** - 所有 58 个任务全部完成
✅ **Final Wave 通过** - 所有验证任务 APPROVE
✅ **可以交付** - 项目已达到生产就绪状态
