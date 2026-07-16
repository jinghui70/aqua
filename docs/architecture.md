# aqua 架构决策(技术架构 Source of Truth)

日期: 2026-07-11 · 状态: 定稿(基于 grill-me 访谈 Q1-Q11)

## 与 design.md 的关系
- `docs/design.md` 是**业务设计** source of truth(数据模型/逻辑类型/DDL 规则/工作流/功能边界/UI 需求),与语言无关,保留。
- `docs/design.md` 的**技术栈章节(§2 技术栈 / §8 打包)已过时**(原 Electron+Node+Java),由本文档取代。
- 本文档是**技术架构** source of truth。两者冲突时:技术架构以本文档为准,业务以 design.md 为准。

## 1. 整体架构
aqua v2 = **Tauri 2.x 桌面 + Rust 后端(`aqua-core`) + Vue3/element-plus 前端 + Java connector(复用)**。纯桌面(无 Web/Docker)。推倒重写 core+Tauri,connector 复用,前端迁移。

```
┌──────────────────────────────────────────────┐
│  src-tauri (Tauri 2.x 壳, Rust)               │
│   main.rs: 无 args 开 GUI / 有 args 走 CLI    │
│   commands: 调 aqua-core + spawn connector    │
├──────────────────────────────────────────────┤
│  crates/aqua-core (Rust 纯逻辑核心)            │
│   schema / generators / dataset / import /    │
│   driver (trait Driver)                       │
├──────────────────────────────────────────────┤
│  app (Vue3 + element-plus, 系统 webview)      │
│   IPC: Tauri invoke                           │
├──────────────────────────────────────────────┤
│  connector (Java, 复用)    drivers/ (用户提供) │
│   一次性命令, stdin/stdout JSON               │
│   MySQL/PG 走 Rust native, 不经 connector     │
└──────────────────────────────────────────────┘
```

## 2. 连接层
- **一次性命令模式**(不常驻):aqua 是低频设计工具,每次 spawn `java -jar connector`,stdin JSON -> stdout JSON -> exit。JVM 冷启动可接受(与 dbx 常驻 daemon 不同)。
- **混合驱动**:
  - MySQL/PG:Rust native(`mysql_async`/`tokio-postgres`),**免 Java**。
  - Oracle/DM/KingBase/GBase/H2:Java JDBC(spawn connector)。
- **Driver trait**:`trait Driver { test_connection / list_tables / get_columns / list_indexes / query_rows }`,native + Jdbc 两实现,返回统一 aqua schema 类型,反解藏各实现内。工厂 `create_driver(config) -> Box<dyn Driver>`。native 一次性连接不维护池。
- **不做 SSH**:直连,用户自保证库可达(VPN/端口转发)。

## 3. 反解(物理类型 -> 逻辑类型)分库归属
- **Java 库**(Oracle/信创/H2):反解在 Java 侧(`Dialect` 子类 `mapType`)。外置 **JDBC 驱动 jar** 经 URLClassLoader 加载(`drivers/databases.json` 记录 installed 的 driverJar/driverClass);driverClass 硬编码在 `dialects.rs` ALL_DATABASES + `DialectRegistry.java`(两处人工同步)。加库:写 Dialect 子类(或复用 `GenericJdbcDialect` 兜底)+ 两处注册 + 用户备 jar。
- **Native 库**(MySQL/PG):反解写死 Rust 各驱动模块(成熟稳定,不常变)。
- 两套反解分库归属,各藏其连接层实现内,trait 返回统一逻辑类型。

## 4. 两类数据库支持机制(核心设计原则)

### 4.1 内置方言 (Native Dialects)
- **定义**: 有成熟 Rust 异步驱动的数据库
- **当前支持**: MySQL(mysql_async), PostgreSQL(tokio-postgres)
- **实现方式**:
  - 反解逻辑硬编码在 Rust 代码中
  - 类型映射硬编码(逻辑类型 ↔ 物理类型)
  - DDL 生成规则硬编码
  - 性能最优,无跨进程通信开销
- **扩展方式**: 修改 Rust 代码,重新编译

### 4.2 JDBC 方言 (JDBC Dialects)
- **定义**: 通过 JDBC 驱动访问的数据库
- **支持范围**: Oracle, SQL Server, DB2, H2, **所有信创数据库**(达梦DM/人大金仓KingBase/南大通用GBase/神通Oscar)等所有有 JDBC 驱动的数据库
- **实现方式**:
  - 通过 `connector.jar` 外置 Java 进程统一访问
  - 反解逻辑在 Java 侧实现(spawn 一次性子进程,stdin/stdout JSON 通信)
  - 类型映射配置外置(在 connector.jar 注册,Rust 侧提供参考实现)
  - DDL 生成规则可外置配置
- **扩展方式**:
  1. 在 connector.jar 添加新的 Dialect 实现类(或复用 `GenericJdbcDialect` 兜底,免写子类)
  2. 在 `DialectRegistry.java` register + `dialects.rs` ALL_DATABASES 加项(driverClass 等,两处人工同步)
  3. 提供 JDBC 驱动 jar(用户自备,经应用内 install_driver 写入 `databases.json`)
  4. 在 Rust 侧添加类型映射参考(可选)
  5. **无需重新编译 aqua**

### 4.3 设计原则

1. **内置优先**: MySQL/PostgreSQL 因使用频率高且有成熟 Rust 驱动,作为内置实现
2. **JDBC 统一**: 其他数据库(包括所有信创库)统一走 JDBC 机制,避免为每个数据库硬编码
3. **配置外置**: JDBC 方言的类型映射/反解规则尽量外置配置,降低扩展成本
4. **开发聚焦**: 开发时不执着于具体信创数据库,关注机制本身(JDBC Dialect 扩展机制)
5. **用户扩展**: 新增数据库支持无需aqua源码修改,用户可自行扩展

## 5. 打包分发
- `connector.jar`:**内置打包**(~10MB,含连接/元数据/反解框架,不含 JDBC 驱动)。
- JRE:**用户自备 JDK 17+**(不内置不下载)。用 Java 库前用户自装;MySQL/PG 免 Java。connector 代码须兼容 Java 17 语法。
- JDBC 驱动 jar:用户提供(运行时落 `app_data_dir/drivers/`,经应用内 install_driver 写入 `databases.json`;driverClass 在 `dialects.rs`/`DialectRegistry.java` 硬编码)。
- 安装包 **~20MB**(Tauri+connector+前端,无 JRE)。原 design.md 170MB -> ~20MB。
- **不做自动更新**。**仅中文**。app 需检测 JDK >=17(用 Java 库场景)。

## 6. 前端
- 放弃 json-ui,用 **element-plus**(为了快,免学 json-ui)。design.md "全 json-ui 渲染"修正为 Vue3+element-plus 手写组件。
- 前端 JSON 生成(`generators/frontend-json`)**保留移植**,服务于外部 json-ui 项目(非 aqua 自身 UI)。
- Tauri = Rust 后端 + Web 前端(系统 webview)。前端必然 Web 技术(无"Rust 写前端")。

## 7. CLI
- Tauri 二进制**双模式**:无 args 开 GUI,有 args(`aqua generate ...`)走 CLI 不开窗。generate 逻辑在 aqua-core。
- generator 内置 4 个(ddl/java/json/strconst),扩展靠改程序重编译(plugin 机制延后)。

## 8. 项目结构
```
aqua/
  Cargo.toml              # workspace: crates/aqua-core, src-tauri
  crates/aqua-core/       # Rust 纯逻辑核心(schema/generators/dataset/import/driver)
  src-tauri/              # Tauri 壳 + CLI 入口
  app/                    # Vue3/element-plus 前端
  connector/              # Java connector(复用,反解移入外置 jar)
  drivers/                # JDBC 驱动 jar(用户提供,运行时落 app_data_dir/drivers/)
  docs/                   # design.md(业务) + architecture.md(本文) + grill-me 记录
```

## 9. 迁移策略
- **推倒重写** core+Tauri(原 TS 版在 `~/work/aqua-legacy` 作参考蓝本,不直接迁代码)。
- connector Java 复用(反解从 core 移入 Java 外置 jar)。
- 前端 Vue 迁移(IPC 从 Electron 改 Tauri invoke)。
- **移植起点**:`schema`(数据模型基础)-> `generators`(纯逻辑快速验证)-> `dataset` -> `import`(依赖 Driver trait)-> `diff/ALTER`(新功能)。

## 10. 决策索引(grill-me Q1-Q11)
- Q1 一次性命令 + 反解移 Java
- Q2 反解 Java 侧外置 JDBC 驱动 jar(URLClassLoader + databases.json)
- Q3 混合架构 + 反解分库归属
- Q4 trait Driver 收敛(native + Jdbc 两实现) + **两类数据库支持机制**
- Q5 纯桌面 + 推倒重写 + 目标结构
- Q6 用户自备 JDK 17+ + connector 内置 + 驱动用户提供
- Q7 放弃 json-ui 用 element-plus + 前端 JSON 生成保留
- Q8 CLI Tauri 双模式 + generator 内置
- Q9 不做 SSH 直连
- Q10 不自动更新 + 仅中文 + 移植起点 schema
- Q11 新建 ~/work/aqua,原目录 mv aqua-legacy

完整访谈记录:`docs/grill-me-2026-07-11.md`。原始 grill-me 记录存 `~/study/dbx/brainstorms/2026-07-11-rust-return-evaluation.md`。
