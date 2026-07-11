# aqua v2 开发路线图

**状态**: 2026-07-12 · 任务规划完成  
**已完成**: 2 个任务  
**进行中**: 0 个任务  
**待开始**: 20 个任务

---

## 已完成 ✅

1. **00-bootstrap-guidelines** (已归档) - 项目编码规范填充
2. **07-11-schema-migration** (已归档) - schema 模块移植至 Rust

---

## P0 优先级 (阻塞后续,优先实现)

这些任务验证核心架构可行性,完成后其他模块可并行推进:

1. **07-12-generators-ddl** - DDL 生成器 (7 方言)
   - 验证 schema 模型正确性
   - 生成 CREATE TABLE/INDEX
   - 类型映射(逻辑类型 → 物理类型)

2. **07-12-generators-java** - Java 实体生成器
   - rainbow-dbaccess 注解
   - 验证业务类型系统

3. **07-12-driver-trait** - Driver trait 定义
   - test_connection/list_tables/get_columns/list_indexes
   - 工厂模式 create_driver

4. **07-12-driver-mysql** - MySQL native 驱动
   - mysql_async 实现
   - 反解 MySQL → 逻辑类型

---

## P1 核心功能 (基础可用版本)

完成这些任务后 aqua 可进入基础可用状态(GUI 编辑 + 生成 DDL/Java + 从 MySQL 导入):

### aqua-core

5. **07-12-import-module** - 导入模块
   - 调用 Driver trait
   - 生成 Project JSON

### src-tauri

6. **07-12-cli-mode** - CLI 模式
   - 参数解析 (clap)
   - `aqua generate --type ddl --dialect mysql`

7. **07-12-tauri-commands-project** - 项目管理 commands
   - project_open/project_save/project_validate

8. **07-12-tauri-commands-generate** - 生成器 commands
   - generate_ddl/generate_java

9. **07-12-tauri-commands-import** - 导入 commands
   - import_from_db/test_connection

### app (前端)

10. **07-12-frontend-editor** - 项目编辑器主界面
    - 左侧表树 + 右侧属性面板

11. **07-12-frontend-table-field** - 表与字段编辑组件
    - 表名/分组/字段列表
    - dataType/length/precision/isKey/notNull

12. **07-12-frontend-generator-ui** - 生成器配置 UI
    - DDL 方言选择
    - Java 包名配置
    - 预览窗口

13. **07-12-frontend-import-wizard** - 导入向导
    - 数据库连接配置
    - 表选择
    - 进度显示

---

## P2 增强功能 (完整功能集)

这些任务补全剩余功能,达到 feature complete:

### aqua-core

14. **07-12-generators-frontend-json** - 前端 JSON 生成器
    - json-ui 兼容格式

15. **07-12-generators-strconst** - StrConst 生成器
    - 数据库常量类

16. **07-12-diff-engine** - diff 引擎
    - Project vs Project 对比

17. **07-12-alter-generator** - ALTER DDL 生成
    - 基于 diff 结果
    - 7 方言支持

18. **07-12-dataset-sqlite** - dataset 模块
    - SQLite 容器实现
    - schema.json + 数据表打包

19. **07-12-driver-postgres** - PostgreSQL native 驱动
    - tokio-postgres + deadpool

20. **07-12-driver-jdbc** - JDBC 驱动 (spawn connector.jar)
    - Oracle/DM/KingBase/GBase/H2

---

## P3 优化与完善 (待创建)

- 单元测试覆盖 (generators/diff/driver)
- 集成测试 (端到端 GUI/CLI)
- 前端其他 UI (Enum 管理/BizType 管理/Diff UI/数据集 UI)
- 打包与发布 (Tauri bundle/connector.jar/文档)

---

## 依赖关系

```
schema (已完成)
  ├─> generators-ddl (P0)
  │     ├─> cli-mode (P1)
  │     ├─> tauri-commands-generate (P1)
  │     └─> frontend-generator-ui (P1)
  ├─> generators-java (P0)
  ├─> generators-frontend-json (P2)
  ├─> generators-strconst (P2)
  ├─> diff-engine (P2)
  │     └─> alter-generator (P2)
  └─> driver-trait (P0)
        ├─> driver-mysql (P0)
        │     ├─> import-module (P1)
        │     │     ├─> tauri-commands-import (P1)
        │     │     └─> frontend-import-wizard (P1)
        │     └─> dataset-sqlite (P2)
        ├─> driver-postgres (P2)
        └─> driver-jdbc (P2)

tauri-commands-project (P1)
  └─> frontend-editor (P1)
        └─> frontend-table-field (P1)
```

---

## 建议实施顺序

### Sprint 1: 验证核心架构 (P0)
1. generators-ddl
2. generators-java
3. driver-trait
4. driver-mysql

**里程碑**: CLI 可生成 DDL/Java,验证 schema → 代码全链路

### Sprint 2: GUI 基础 (P1 前端 + Tauri)
5. tauri-commands-project
6. frontend-editor
7. frontend-table-field
8. tauri-commands-generate
9. frontend-generator-ui

**里程碑**: GUI 可编辑项目 + 生成 DDL/Java

### Sprint 3: 导入功能 (P1 导入链路)
10. import-module
11. cli-mode (含 import 命令)
12. tauri-commands-import
13. frontend-import-wizard

**里程碑**: 可从 MySQL 导入存量结构,aqua 基础可用

### Sprint 4: 完整功能集 (P2)
14-20. 剩余 P2 任务并行推进

**里程碑**: feature complete,进入测试与优化阶段
