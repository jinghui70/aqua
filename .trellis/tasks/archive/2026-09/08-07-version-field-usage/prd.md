# aqua 文件 version 字段用途

## Goal
确定 `.aqua` 项目文件 `Project.version` 字段的语义与消费方式:该用于什么、是否需要消费点,或弃用。
(前置:已把 newProject 硬编码 "1.0.0" 改为 `APP_VERSION` 构建期注入,但"填了干嘛"未定。)

## Background
- `version: string` 存在于 Project schema:`docs/design.md:239` 标注"产品版本号",Rust `crates/aqua-core/src/schema/project.rs:18`,TS `app/src/types/schema.ts:102`。
- `.aqua` 文件经 Tauri 命令 `project_open`/`project_save` 直接 serde JSON 读写(`src-tauri/src/commands/project.rs:7-20`),version 随 Project 一起序列化,无任何转换。
- **Rust 端与前端均无任何代码读取 version 做决策**(grep 全仓无消费点)。字段当前为死字段:写进文件,无人读。
- schema 演进策略是"反序列化容错",不依赖 version:可选字段 + serde default。证据:`name?`(旧 schema 兼容, `project.rs:19`)、`biz_type.rs:70` 旧 schema 无 default 字段、`table.rs:28` index direction 兼容旧纯字符串。

## Confirmed Facts
- version 字段语义在设计文档中仅标注"产品版本号",未说明用途。
- 当前 schema 演进不需要 version(靠反序列化容错已 work)。
- version 已随文件持久化,但无读取消费点 -> 当前是技术债/死字段。
- 已改:`newProject` 填 `APP_VERSION`,`openProject` 用文件原值,`saveProject` 保存当前内存值(旧文件保存后 version 不变)。

## Decisions
1. version 语义 = 创建/最后编辑此文件的 aqua 产品版本(非数据 schema 版本)。
2. 用途 = 兼容性保护(原选项 A 的具体化):
   - 低版本 app 打开高版本文件(version > app 版本) -> 拒绝打开,提示升级 app。
   - 高版本 app 打开低版本文件(version < app 版本) -> 允许打开(反序列化容错保证可读,schema 是旧版超集)。

## Open Questions (blocking)
1. 高版本 app 打开低版本文件,**保存时 version 是否升级为当前 app 版本**?(决定 version = 最后编辑版本 vs 创建版本,影响低打高拒绝逻辑的自洽性)
2. 高打低打开时是否给一次性轻量提示?(UX 偏好,次要)

## Notes
- 探索证据见 Background;实现前需先定用途。
