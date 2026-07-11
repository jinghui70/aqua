# aqua v2 开发任务规划

基于 architecture.md 移植路线 + design.md 功能清单

## Phase 1: 核心逻辑层 (aqua-core)

### 1.1 数据模型 (已完成)
- [x] schema 模块移植 (Task: 07-11-schema-migration, 已归档)
  - DataType/Field/Table/Project/BizType/Enum
  - validate/parse_project
  - 测试 + fixtures

### 1.2 生成器 (generators)
- [ ] DDL 生成器 (7 方言)
  - MySQL/PostgreSQL/Oracle/DM/KingBase/GBase/H2
  - 从 Project 生成 CREATE TABLE/INDEX
  - 类型映射(逻辑类型 → 物理类型)
- [ ] Java 实体生成器
  - rainbow-dbaccess 注解(@Table/@Column/@Index)
  - 驼峰命名/Lombok/序列化
- [ ] 前端 JSON 生成器
  - json-ui 兼容格式(外部项目用)
- [ ] StrConst 生成器
  - 数据库常量类(表名/字段名)

### 1.3 Driver trait 与连接层
- [ ] Driver trait 定义
  - test_connection/list_tables/get_columns/list_indexes/query_rows
- [ ] MySQL native 驱动实现
  - mysql_async
  - 反解 MySQL 物理类型 → 逻辑类型
- [ ] PostgreSQL native 驱动实现
  - tokio-postgres + deadpool
  - 反解 PG 物理类型 → 逻辑类型
- [ ] JDBC 驱动实现 (spawn connector.jar)
  - stdin/stdout JSON 通信
  - Oracle/DM/KingBase/GBase/H2 统一接口

### 1.4 数据集 (dataset)
- [ ] SQLite 容器实现
  - schema.json + 数据表打包为 .aqua 文件
  - 读写 API
- [ ] JSON 格式数据集
  - schema.json + data/*.json
- [ ] 数据集导入/导出
  - 库 → 数据集 → 库迁移

### 1.5 导入 (import)
- [ ] 从数据库导入 schema
  - 调用 Driver trait
  - 生成 Project JSON
  - 反解完整性(表/字段/索引/注释)

### 1.6 Diff 与 ALTER (新功能)
- [ ] JSON diff 引擎
  - Project vs Project 对比
  - 检测表/字段/索引变更
- [ ] ALTER DDL 生成
  - ADD/DROP/MODIFY COLUMN
  - ADD/DROP INDEX
  - 7 方言支持

## Phase 2: Tauri 壳层 (src-tauri)

### 2.1 CLI 模式
- [ ] 参数解析 (clap)
  - `aqua generate --type ddl --dialect mysql`
  - `aqua generate --type java`
  - `aqua import --from mysql --output schema.json`
- [ ] CLI 命令实现
  - 调用 aqua-core generators
  - stdout 输出结果

### 2.2 Tauri Commands
- [ ] 项目管理 commands
  - project_open/project_save/project_validate
- [ ] 生成器 commands
  - generate_ddl/generate_java/generate_json/generate_strconst
- [ ] 导入 commands
  - import_from_db (spawn connector)
  - test_connection
- [ ] Diff commands
  - diff_projects/generate_alter

### 2.3 Java Connector 集成
- [ ] 检测 JDK 版本 (>=17)
- [ ] spawn connector.jar
- [ ] stdin/stdout JSON 通信
- [ ] 错误处理与映射

## Phase 3: 前端 (app)

### 3.1 核心 UI 组件
- [ ] 项目编辑器主界面
  - 左侧树(表/字段层级)
  - 右侧属性面板
- [ ] 表编辑组件
  - 表名/分组/注释
  - 字段列表(增删改)
- [ ] 字段编辑组件
  - dataType/length/precision/scale
  - isKey/notNull/defaultValue
  - bizType/enum(全局引用/内联)
  - autoGenerate 配置
- [ ] 索引编辑组件
  - 字段选择(多选)
  - unique 标记

### 3.2 业务类型 UI
- [ ] 内置 bizType 配置面板
- [ ] 自定义 bizType 定义
- [ ] bizTypeData 表单生成

### 3.3 Enum UI
- [ ] 全局 Enum 管理
  - code/name/package
  - hasCode 开关
  - values 列表(id/name/code/color)
- [ ] 内联 Enum 编辑器

### 3.4 生成器 UI
- [ ] DDL 生成配置
  - 方言选择(7 种)
  - 输出路径
  - 预览窗口
- [ ] Java 生成配置
  - 包名/类名前缀
  - Lombok 开关
- [ ] 前端 JSON 生成配置
- [ ] StrConst 生成配置

### 3.5 导入向导
- [ ] 数据库连接配置
  - host/port/user/password/database
  - 连接测试
- [ ] 表选择(多选)
- [ ] 导入进度显示

### 3.6 Diff 与 ALTER UI
- [ ] 版本选择(JSON 文件对比)
- [ ] Diff 结果展示
  - 新增/删除/修改表
  - 新增/删除/修改字段
  - 新增/删除索引
- [ ] ALTER DDL 预览与导出

### 3.7 数据集 UI
- [ ] 数据集定义
  - 选择表
  - 配置数据行数/过滤条件
- [ ] 数据集导入/导出
  - 从库导入
  - 导出为 INSERT/SQLite

## Phase 4: 测试与质量

### 4.1 单元测试
- [ ] aqua-core 各模块测试覆盖
  - generators 输出验证
  - diff 算法正确性
  - Driver trait 各实现

### 4.2 集成测试
- [ ] 端到端测试
  - GUI: 打开 → 编辑 → 生成 → 保存
  - CLI: generate 命令输出验证
- [ ] 数据库连接测试
  - MySQL/PG/Oracle 真实库测试
  - JDBC connector 通信测试

### 4.3 质量门禁
- [ ] clippy -D warnings 全项目
- [ ] cargo fmt --check
- [ ] 前端 TS 类型检查 (tsc --noEmit)

## Phase 5: 打包与发布

### 5.1 构建流程
- [ ] Tauri 打包配置
  - macOS/Windows/Linux 三平台
  - 应用图标/签名
- [ ] connector.jar 打包
  - fat jar(含反解框架,不含驱动)

### 5.2 文档
- [ ] 用户手册
  - 安装指南(含 JDK 17+ 要求)
  - 功能演示
- [ ] 开发者文档
  - 架构说明
  - 贡献指南

### 5.3 发布
- [ ] GitHub Release
- [ ] 版本管理(语义化版本)

## 优先级说明

**P0** (阻塞后续):
- generators (DDL/Java) - 验证 schema 正确性的最快路径
- Driver trait + MySQL native - 导入功能基础

**P1** (核心功能):
- 前端编辑器主界面
- Tauri commands (project/generate)
- CLI 模式

**P2** (增强功能):
- Diff + ALTER
- 数据集管理
- 其他方言驱动 (PG/JDBC)

**P3** (优化):
- 测试覆盖
- 文档完善
