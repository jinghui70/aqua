# 数据库结构管理工具 - 详细开发设计

**版本**: v3.0  
**日期**: 2026-07-08  
**状态**: 需求与架构定稿（Q1-Q37），待开发  
**基础**: 访谈记录 `brainstorms/2026-07-07-database-schema-tool.md`

---

## 1. 项目定位

### 1.1 核心目标

**保证数据结构的唯一性，作为系统开发依据。**

- JSON 是 Single Source of Truth (SSOT)
- DDL、代码、前端数据模型均从 JSON 派生
- **这是数据库设计工具，不是管理工具**——产出 DDL/ALTER 文本，永不自动执行

### 1.2 核心能力

1. **表结构建模**：定义表/字段/索引/分组，细粒度逻辑类型
2. **业务类型系统**：跨前后端共享语义层
3. **Enum 系统**：全局/内联枚举，特殊业务类型
4. **DDL 生成**：7 种方言（MySQL/PG/Oracle/DM/KingBase/GBase/H2）
5. **代码生成**：Java 实体（rainbow-dbaccess 注解）+ 前端 JSON（json-ui 兼容）+ StrConst
6. **diff + ALTER**：JSON 版本对比，生成 ALTER DDL
7. **导入**：连库读结构（Java JDBC，6 种数据库）
8. **数据集管理**：JSON/SQLite 双格式，库↔数据集↔库迁移
9. **CLI**：generate（统一命令，plugin 化 type）

### 1.3 工作流

**新项目**:
```
定义 JSON -> 生成 DDL 建库 -> 生成代码 -> 定义数据集
```

**存量接管**（一次性反向）:
```
连库导入 -> 生成 JSON -> 此后 JSON 为主
```

**日常演进**:
```
改 JSON -> diff(JSON vs JSON) -> 生成 ALTER -> 人工执行 -> 重生成代码
```

**数据迁移**:
```
库A -> 导入到数据集 -> 导出 INSERT -> 库B 执行
```

**单元测试**:
```
JSON -> aqua generate --type ddl --dialect h2 + 数据集 -> 初始化内存库
```

---

## 2. 技术栈

### 2.1 核心架构

**混合架构：Electron + Node 主进程 + Java 子进程**

- 核心逻辑（DDL/diff/代码生成）用 TypeScript（与前端一致）
- 数据库连接用 Java + JDBC（统一接口，支持所有数据库含信创）
- Oracle 不需 Instant Client（用 ojdbc.jar）
- 信创数据库全支持（DM/KingBase/GBase）

```
Electron 主进程 (Node/TypeScript)
  ├─ 文件系统操作（项目读写）
  ├─ 核心逻辑库（纯 TS）
  │   ├─ DDL 生成引擎（7 方言）
  │   ├─ ALTER 生成引擎（从 diff）
  │   ├─ diff 引擎（JSON vs JSON）
  │   ├─ 代码生成（Java 实体 / 前端 JSON / StrConst）
  │   ├─ 业务类型引擎
  │   └─ Enum 引擎
  └─ 数据库连接时：spawn('java', ['-jar', 'db-connector.jar', ...])

渲染进程 (Vue3 + json-ui)
  ├─ 全部页面用 JSON 写（json-ui 的 JsonRender 渲染）
  ├─ Vue Router 路由
  └─ 通过 IPC 调用主进程能力

Java CLI (db-connector.jar)
  ├─ JDBC 统一连接所有数据库
  ├─ 命令：import / query
  ├─ 所有 JDBC 驱动打包进 fat jar
  └─ 输出 JSON 到 stdout
```

### 2.2 数据库支持

| 数据库 | JDBC 驱动 | 说明 |
|---|---|---|
| MySQL | mysql-connector-j | 官方 |
| PostgreSQL | postgresql | 官方 |
| Oracle | ojdbc11 | 无需 Instant Client |
| 达梦 DM | dm8-jdbc | 信创 |
| 人大金仓 KingBase | kingbase8 | 信创 |
| 南大通用 GBase | gbase-connector-java | 信创 |
| H2 | - | DDL 生成兼容模式（不直连） |

### 2.3 资源打包

```
app/resources/
  jre/                    # 精简 JRE (jlink 裁剪, ~40MB)
  db-connector.jar        # Java 连接器 + JDBC 驱动 (~10MB)
```

**精简 JRE**（jlink）：
```bash
jlink --add-modules java.base,java.sql,java.naming,java.xml \
  --strip-debug --no-header-files --no-man-pages --compress=2 \
  --output jre-headless
```

**安装包体积**：~170MB（Electron 120MB + JRE 40MB + jar 10MB）

### 2.4 前端技术栈

- Vue 3 + Vue Router + json-ui + ElementPlus + UnoCSS + TypeScript
- **json-ui 已发 npm 私服**（localhost:4873，配置 .npmrc）
- **全部路由页面用 JSON 写**（json-ui 的 JsonRender 渲染）
- 包管理：pnpm

---

## 3. 数据模型

### 3.1 逻辑类型（dataType）

| 类型 | 属性 | 说明 |
|---|---|---|
| VARCHAR | length | 变长字符串 |
| CLOB | - | 长文本 |
| TINYINT | - | 小整数（-128~127） |
| INT | - | 32位整数 |
| LONG | - | 64位整数 |
| DECIMAL | precision, scale | 精确小数 |
| DATE | - | 日期 |
| DATETIME | - | 日期时间 |
| BLOB | - | 二进制 |

**不含**：BOOLEAN（跨库不一致，用 TINYINT/VARCHAR+业务类型）、JSON（用 BLOB/CLOB）、DOUBLE（DECIMAL 足够）

### 3.2 字段模型（Field）

完全对齐 json-ui 的 DataFieldSchema + 工具扩展：

```typescript
interface Field {
  // 基础（对齐 json-ui）
  prop: string                // Java/TS 属性名（驼峰，如 userName）
  code: string                // 数据库字段名（大写蛇形，如 USER_NAME）
  name: string                // 中文名（DDL COMMENT）
  dataType: DataType          // 逻辑类型（9 种）
  
  // 类型属性
  length?: number             // VARCHAR 长度
  precision?: number          // DECIMAL 精度（工具扩展）
  scale?: number              // DECIMAL 小数位
  
  // 业务类型（对齐 json-ui）
  bizType?: string            // 业务类型 code
  bizTypeData?: unknown       // 业务类型配置（单 field 直接存值，多 field 存对象）
  
  // 约束（对齐 json-ui）
  isKey?: boolean             // 主键
  notNull?: boolean           // 非空
  
  // 工具扩展
  defaultValue?: string       // DDL DEFAULT 子句
  
  // 应用层生成（对齐 @GeneratedValue）
  autoGenerate?: {
    enabled: boolean
    strategy: "default"|"now"|string  // default=雪花, now=当前时间, 自定义
    param?: string                    // 策略参数（如 "USR_" / "yyyy-MM-dd"）
    timing: "INSERT"|"INSERT_UPDATE"  // 生成时机
  }
  
  // Enum（特殊业务类型）
  enum?: string | InlineEnum  // string=引用全局枚举 code，object=内联枚举
  
  // 文档
  comment?: string            // 详细描述（设计文档用，不进 DDL）
}
```

**规则**：
- `prop` 与 `code`：新建字段时从 code 自动派生 prop（蛇形→驼峰），可手动修改
- `enum` 只支持 VARCHAR

### 3.3 表模型（Table）

```typescript
interface Table {
  code: string                // 表名（大写蛇形）
  name: string                // 中文名（DDL COMMENT）
  group: string               // 引用分组 code（一表一分组）
  fields: Field[]             // 字段列表
  indexes?: Index[]           // 索引
  comment?: string            // 详细描述（设计文档用）
}

interface Index {
  name?: string               // 索引名（空则自动生成）
  fields: string[]            // 字段 code 列表
  unique: boolean             // 唯一索引
}
```

### 3.4 业务类型（BizTypeDefine）

```typescript
interface BizTypeDefine {
  bizType: string             // code（如 "Date8"）
  name: string                // 中文名
  description?: string        // 说明
  
  // 支持的数据类型配置（必需）
  supportedDataTypes: Array<{
    dataType: DataType
    defaultLength?: number      // VARCHAR 默认长度
    defaultPrecision?: number   // DECIMAL 默认精度
    defaultScale?: number       // DECIMAL 默认小数位
  }>
  
  // 业务类型参数配置（前端表单生成用）
  bizTypeData?: {
    fields: Array<{
      name: string
      type: "string" | "number"  // 简化：只有 string/number
      description?: string
      required?: boolean
    }>
  }
}
```

**内置业务类型**：
- 产品打包配置文件，启动加载
- 项目不可删改，升级产品才变

**自定义业务类型**：
- 项目独享，可增删改
- 无版本管理，改动自动检查并提示

**用户交互**：
- 先选数据类型 → bizType 下拉只显示支持该类型的业务类型 → 自动填充默认值
- 先选业务类型 → dataType 下拉只显示该类型支持的类型 → 自动填充默认值

**单 field 简化保存**：
- bizTypeData 只有一个 field 时，字段实例的 bizTypeData 直接保存值（非对象）
- 多 field 时保存对象

### 3.5 Enum（特殊业务类型）

**EnumDefine（全局枚举，schema.json 顶层 enums 数组）**：

```typescript
interface EnumDefine {
  code: string                // 枚举标识（如 "EnumGender"）
  name: string                // 中文名（如 "性别"）
  package: string             // 相对子路径，拼到 basePackage 下
  hasCode?: boolean           // true=CodeEnum 派生存 code，false/无=普通枚举存 id
  values: EnumValue[]
}

interface EnumValue {
  id: string                  // 枚举项标识（"MALE"），Java 枚举名，前端用
  name: string                // 中文显示（"男"）
  code?: string               // 存储值（"M"），hasCode=true 时必填
  color?: string              // 预置颜色（写死代码）
}
```

**color 预置列表**（写死代码，改需改代码）：
```
success / error / warning / info / primary / danger
red / orange / yellow / green / blue / purple / grey
```

**hasCode 行为**：
- `hasCode=true`：数据库存 code，Java 生成 `MALE("M","男") implements CodeEnum`
- `hasCode=false`：数据库存 id，Java 生成普通枚举 `MALE // 男`
- 校验：hasCode=true 时每个 value 必须有 code；hasCode=false 时 code 忽略

**字段引用枚举**（二选一）：
- 引用全局：`field.enum = "EnumGender"`（string）
- 内联：`field.enum = {name, hasCode, values}`（object，无 code/package）

**Java 生成**：
- 全局枚举：生成独立 enum 类（自身 package）
- 内联枚举：生成独立 enum 类（共享表的 package）

**字段配置页交互**：
- bizType = "Enum" → 展示特殊配置页（选引用/内联 → 配置）
- 其他 bizType → 展示 fields 配置页（bizTypeData.fields）

### 3.6 项目结构

**项目 = 目录**，文件命名用项目名前缀：

```
myproject/                        # 项目目录
  myproject.json                  # 主体（项目名.json）
  myproject.test.json             # JSON 数据集（入 Git）
  myproject.dev.db                # SQLite 数据集（.gitignore）
  .dbconfig.json                  # 数据源配置（.gitignore）
  .gitignore
```

**.gitignore**：
```
*.db
.dbconfig.json
```

**schema.json 顶层**：

```typescript
interface Project {
  version: string             // 产品版本号
  basePackage: string         // 全局根 package（如 "com.example"）
  bizTypes: BizTypeDefine[]   // 业务类型（内置注入 + 自定义）
  enums: EnumDefine[]         // 全局枚举
  groups: GroupDefine[]       // 分组（显式定义，数组顺序即排序）
  tables: Table[]             // 表结构
}

interface GroupDefine {
  code: string                // 分组标识（如 "order"）
  name: string                // 中文名（如 "订单模块"）
}
```

**分组规则**：
- 单层分组
- 显式定义（groups 数组），无 sort，数组顺序即排序
- 一表一分组（table.group 引用单个 code）

---

## 4. 核心功能

### 4.1 DDL 生成

**输入**：Project + 目标方言（MySQL/PG/Oracle/DM/KingBase/GBase/H2）  
**输出**：建表 SQL 文本（CREATE TABLE + INDEX + 可选数据集 INSERT）  
**规则**：表名/字段名/关键字**大写**，COMMENT 用 name 字段

#### 逻辑类型 → 物理类型映射

| 逻辑类型 | MySQL | PostgreSQL | Oracle | DM | KingBase | GBase | H2 |
|---|---|---|---|---|---|---|---|
| VARCHAR(n) | VARCHAR(n) | VARCHAR(n) | VARCHAR2(n) | VARCHAR(n) | VARCHAR(n) | VARCHAR(n) | VARCHAR(n) |
| CLOB | TEXT | TEXT | CLOB | CLOB | TEXT | TEXT | CLOB |
| TINYINT | TINYINT | SMALLINT | NUMBER(3) | TINYINT | SMALLINT | TINYINT | TINYINT |
| INT | INT | INTEGER | NUMBER(10) | INT | INTEGER | INT | INT |
| LONG | BIGINT | BIGINT | NUMBER(19) | BIGINT | BIGINT | BIGINT | BIGINT |
| DECIMAL(p,s) | DECIMAL(p,s) | NUMERIC(p,s) | NUMBER(p,s) | DECIMAL(p,s) | NUMERIC(p,s) | DECIMAL(p,s) | DECIMAL(p,s) |
| DATE | DATE | DATE | DATE | DATE | DATE | DATE | DATE |
| DATETIME | DATETIME | TIMESTAMP | TIMESTAMP | TIMESTAMP | TIMESTAMP | DATETIME | TIMESTAMP |
| BLOB | BLOB | BYTEA | BLOB | BLOB | BYTEA | BLOB | BLOB |

#### autoGenerate 不进 DDL

autoGenerate 是应用层生成（@GeneratedValue），DDL 不体现。

#### 数据集 INSERT

- 指定 `--dataset` 时附加 INSERT（标准 SQL，不分方言）
- 表内行按主键排序，表间按数据集数组顺序

### 4.2 代码生成

#### 4.2.1 Java 实体（按 rainbow-dbaccess 规范）

**package 规则**：`{basePackage}.{groupCode}.{entity}`，各段可编辑

**生成规则**：
- 类名：表 code 派生 PascalCase（如 `USER_INFO` → `UserInfo`），可编辑
- 属性名：field.prop（驼峰）
- 类型映射：VARCHAR/CLOB→String，TINYINT/INT→Integer，LONG→Long，DECIMAL→BigDecimal，DATE→LocalDate，DATETIME→LocalDateTime，BLOB→byte[]
- 注解：
  - `isKey=true` → `@Id`
  - `autoGenerate` → `@GeneratedValue(strategy, param, timing)`
  - 非标准命名 → `@Column(name)`
- `comment` → Javadoc
- 枚举字段：生成对应 enum 类（全局/内联）

**配置项**（表编辑页 java Tab）：
- 包名（默认拼好，可编辑）
- 类名（默认派生，可编辑）
- Lombok @Data 开关
- 生成注释开关

#### 4.2.2 前端 JSON（按 json-ui 规范）

**映射规则**：

| 工具字段 | json-ui 字段 | 说明 |
|---|---|---|
| prop | prop | 直接使用 |
| code | code | 直接使用 |
| name | name | 直接使用 |
| INT/LONG/DECIMAL/TINYINT | NUMBER | dataType 粗粒度映射 |
| VARCHAR/CLOB/BLOB | STRING | dataType 粗粒度映射 |
| DATE/DATETIME | DATE/DATETIME | 不变 |
| length/scale | length/scale | 直接使用 |
| bizType/bizTypeData | bizType/bizTypeData | 直接使用 |
| isKey/notNull | isKey/notNull | 直接使用 |
| precision/autoGenerate/comment | - | 不输出 |

#### 4.2.3 StrConst（字符串常量类）

```java
public class StrConst {
    // 表名
    public static final String SYS_USER = "SYS_USER";
    // 字段名
    public static final String USER_ID = "USER_ID";
}
```

**规则**：
- 表名 + 字段名都导出
- **去重**：跨表重复字段名只保留一个
- 范围可选：表 / 分组 / 全部
- package：`{basePackage}.const`（子路径可编辑）
- 类名：默认 `StrConst`，可编辑

### 4.3 diff + ALTER

**diff 输入**：两份 Project JSON（当前 vs 旧版）  
**diff 输出**：结构化差异列表（DiffResult）

```typescript
interface DiffResult {
  tables: {
    added: string[]
    removed: string[]
    changed: Array<{
      table: string
      fields: { added: Field[], removed: string[], changed: Array<{field, changes}> }
      indexes: { added: Index[], removed: string[] }
    }>
  }
  bizTypes: { added, removed, changed }
  enums: { added, removed, changed }
}
```

**ALTER 生成**（从 DiffResult，按方言）：

| 差异类型 | ALTER 语句 |
|---|---|
| 新增表 | CREATE TABLE |
| 删除表 | DROP TABLE |
| 新增字段 | ALTER TABLE ADD COLUMN |
| 删除字段 | ALTER TABLE DROP COLUMN |
| 修改字段 | MySQL: MODIFY COLUMN / PG: ALTER COLUMN TYPE / Oracle: MODIFY |
| 修改默认值 | ALTER TABLE ALTER COLUMN SET DEFAULT |
| 新增索引 | CREATE INDEX |
| 删除索引 | DROP INDEX |

**产物**：可下载 .sql 文件，分步骤注释（便于 DBA 分段执行）

### 4.4 导入（连库读结构）

**流程**：连库 → db-connector 读 JDBC 原始元数据（物理类型+length/precision/scale）→ 主进程加载 resolver JS 配置做反解逻辑类型 → Project JSON

**物理类型 → 逻辑类型反解**（主进程，按方言 resolver JS 配置，按长度/精度推断）：
- MySQL VARCHAR(n) → VARCHAR(n)
- MySQL INT/BIGINT → INT/LONG
- MySQL DECIMAL(p,s) → DECIMAL(p,s)
- MySQL TEXT/LONGTEXT → CLOB
- Oracle NUMBER(p,0) → p≤10:INT, p>10:LONG
- Oracle NUMBER(p,s) s>0 → DECIMAL(p,s)

**resolver 外置**：每个数据库方言的反解逻辑为独立 JS 文件（`resolvers/<type>.js`），主进程动态加载执行。新增数据库：放驱动 jar + resolver JS + registry 加条目，零代码扩展。

**不猜 bizType**（留空，人工标注）

### 4.5 数据集

#### 格式

- **JSON 数据集**（`.json`）：可读，入 Git，小数据量
- **SQLite 数据集**（`.db`）：二进制，不入 Git，大数据量
- 格式从扩展名判断，创建时选

#### SQLite 存储规则（保持精度）

| 逻辑类型 | SQLite 存储 |
|---|---|
| VARCHAR/CLOB | TEXT |
| TINYINT/INT/LONG | INTEGER |
| DECIMAL | TEXT（字符串，避免精度丢失） |
| DATE/DATETIME | TEXT |
| BLOB | BLOB |

#### 数据集 JSON 格式

```json
[
  {
    "table": "USER",
    "data": [
      {"ID": "001", "USER_NAME": "admin", "AGE": 30, "AMOUNT": "99.50", "REMARK": null}
    ]
  }
]
```

- key 用数据库字段名（与表结构 code 一致）
- DECIMAL 用字符串，INT/LONG/TINYINT 用数字
- DATE/DATETIME/VARCHAR/CLOB 用字符串
- BLOB 用 base64
- 空值用 null（不省略 key）

#### 数据集必须匹配项目表结构

- 数据集的表 = 项目 schema.json 的表
- 不支持自由数据

#### 导入（库 → 数据集）

| 模式 | 结构要求 | 映射 |
|---|---|---|
| 单表导入 | 异构可映射 | 支持字段映射（源表与目标表可不同名） |
| 多表/全表导入 | 必须结构一致 | 不映射，直接导，报错提示 |

- **导入只有覆盖，没有追加**

#### 迁移路径

库A → 导入到数据集 → 导出 INSERT → 库B 执行（数据集是中间载体）

---

## 5. 命令行（CLI）

### 5.1 定位

**CLI = 工具本身的命令行模式**（Electron 应用 CLI 入口），供 AI/脚本调用，不是单独的 Java CLI。Java 子进程是内部实现，用户不感知。

### 5.2 命令（统一 generate，plugin 化）

```bash
aqua generate --project ./myproject --type <type> [options]
# 输出到 stdout，用操作系统管道重定向
```

**type 种类**（plugin 机制，可扩展）：

#### type: ddl
```bash
aqua generate --project ./myproject --type ddl \
  [--dialect mysql]              # 默认 mysql
  [--table user,order]           # 可选，逗号分隔
  [--group core]                 # 可选，与 --table 互斥
  [--dataset test]               # 可选，包含数据集（仅 JSON 数据集）
# 无 --table 无 --group -> 全部表
# 不指定 --dataset -> 仅表结构（CREATE TABLE + INDEX）
# 指定 --dataset -> DDL + 数据集 INSERT
```

**方言**：mysql / postgresql / oracle / dm / kingbase / gbase / h2

**--dataset 限制**：只支持 JSON 数据集（SQLite 二进制无法流式输出 stdout）

#### type: java（单表）
```bash
aqua generate --project ./myproject --type java \
  --table user                   # 必需，只支持单表
  [--package com.example.entity] # 可选，默认 basePackage.groupCode.entity
```

#### type: json（单表）
```bash
aqua generate --project ./myproject --type json \
  --table user                   # 必需，只支持单表
```

#### type: strconst
```bash
aqua generate --project ./myproject --type strconst \
  [--table user,order]           # 可选，默认全部
  [--group core]                 # 可选
  [--package com.example.const]  # 可选，默认 basePackage.const
  [--classname StrConst]         # 可选，默认 StrConst
```

### 5.3 plugin 机制

```typescript
interface Generator {
  type: string    // "ddl" | "java" | "json" | "strconst" | ...
  generate(project: Project, options: GenerateOptions): string
}
```

- 内置 generator：ddl / java / json / strconst
- 第三方可注册自定义 generator（如 dao/mapper/typescript）

## 6. UI

### 6.1 菜单栏

```
[文件]  [配置]  [导出]  [帮助]

文件: 新建项目 / 打开项目 / 保存 / 另存为 / 最近项目 / 退出
配置: 业务类型管理 / 数据集管理 / 数据源配置
导出: DDL / diff / StrConst
帮助: 用户指南 / 关于
```

**不做撤销/重做**，防误操作靠：保存确认 + 自动备份(.bak) + Git

### 6.2 主界面

```
[菜单栏]
[工作区]
  ┌─────────────┬──────────────────────────────┐
  │  表列表     │   表编辑区                    │
  │  (侧边栏)   │                              │
  │  📁 核心模块 │  [表基本信息]                 │
  │    ├ USER   │  [fields][index][java][json] │
  │    └ ROLE   │                              │
  │  📁 订单模块 │  Tab 内容...                  │
  │    └ ORDER  │                              │
  │  [+ 新建表] │                              │
  └─────────────┴──────────────────────────────┘
[状态栏]
```

- 表列表：分组树（可折叠/拖拽），支持搜索
- 表/字段支持复制粘贴（Ctrl+C/V，code 加后缀，支持跨项目/跨表）

### 6.3 表编辑页（4 个 Tab）

#### Tab: fields
- 字段表格（列：prop/code/name/dataType/length/bizType/约束...）
- 行内编辑 + 弹窗编辑
- bizType=Enum → 特殊配置页（引用/内联）
- 其他 bizType → fields 配置页
- 拖拽排序、复制粘贴

#### Tab: index
- 索引表格（name/fields/unique）
- 新增/删除

#### Tab: java（只读预览 + 下载）
- 配置项：包名（默认 `{basePackage}.{groupCode}.{entity}`，可编辑）/ 类名 / Lombok / 注释
- 配置改动 → 预览实时刷新
- 操作：复制 / 下载

#### Tab: json（只读预览 + 下载）
- 无配置项，直接输出 json-ui 兼容格式
- 操作：复制 / 下载

### 6.4 数据集管理页（配置菜单）

```
[配置] -> 数据集管理

┌─ 测试数据集管理 ────────────────────────────────────────────┐
│ 数据集: [test ▼]  [新建]  [删除]  [☐ 隐藏无数据表]          │
│                                                            │
│ [表树]                          [数据编辑区]                │
│ ┌────────────────┬──────────────────────────────────────┐ │
│ │ 📁 核心模块      │  USER 表数据                          │ │
│ │  ├ USER (5) ◀  │  [新增行] [从库导入] [导出表数据] [清空] │ │
│ │  └ ROLE (0)    │                                      │ │
│ │ 📁 订单模块      │  ┌────┬──────────┬────┐              │ │
│ │  └ ORDER (10)  │  │ ID │USER_NAME │... │              │ │
│ │                │  └────┴──────────┴────┘              │ │
│ └────────────────┴──────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────┘
```

- 顶部：数据集下拉选择 + 新建 + 删除 + 隐藏无数据表
- 表树：按分组，显示行数（USER(5)）
- 数据编辑区：虚拟滚动（大数据量），新增行/从库导入/导出表数据/清空

#### 从库导入（单表，异构映射）

```
数据源: [dev ▼]
源表: [OLD_USER ▼]（可与目标表不同名）
字段映射:
  ID -> [ID ▼]
  NAME -> [USER_NAME ▼]
  AGE -> [AGE ▼]
WHERE / 行数限制
[导入]（覆盖）
```

#### 从库导入（多表/全表，结构一致）

```
数据源: [dev ▼]
☑ 全部表（或勾选多表）
[导入]（直接按表名匹配，结构不一致报错）
```

#### 导出表数据（单表）
- 选目标方言 → 生成 INSERT 文件

### 6.5 业务类型管理（配置菜单）

- 左侧列表（内置标记不可删，自定义可删）
- 右侧编辑（bizType/name/supportedDataTypes/bizTypeData.fields）

### 6.6 Enum 管理

- 全局枚举列表 + 编辑（code/name/package/hasCode/values[id/name/code?/color?]）
- 字段引用全局枚举或内联

### 6.7 数据源配置（配置菜单）

- 数据源列表（dev/test/prod）
- 新增/编辑/删除
- 表单：type/host/port/database/user/password
- 测试连接
- 存 `.dbconfig.json`（密码加密）

### 6.8 导入向导（文件菜单/导入）

4 步：
1. 选数据源（dev/test/prod/新建）
2. 选表（多选 + 全选/反选/搜索）
3. 导入选项（分组选择）
4. 确认导入（同名表提示：覆盖/跳过）

**导入表结构只导结构，不导数据**

### 6.9 导出（导出菜单，3 项）

所有导出都支持**预览 + 复制 + 下载**。

#### 导出 DDL
```
目标方言: [MySQL ▼]
范围: ○ 全部表  ○ 按分组  ○ 指定表
包含数据集: [无 ▼]（可选）
☐ 仅表结构
[预览]  [下载 .sql]
```

#### 导出 diff
```
对比文件: [选择旧版 schema.json]
输出:
  ○ 差异报告（文本/HTML/Markdown）
  ○ ALTER DDL（选方言）
[生成]  [下载]
```

#### 导出 StrConst
```
范围: ○ 全部表  ○ 按分组  ○ 指定表
包名: [com.example.const]
类名: [StrConst]
[预览]  [下载 StrConst.java]
```

### 6.10 欢迎页

- 新建项目 / 打开项目
- 最近项目列表（存 `~/.local/share/aqua/`）

---

## 7. 数据源配置

**存储**：`.dbconfig.json`（项目目录，进 .gitignore）

```typescript
{
  sources: {
    dev: {
      type: "mysql" | "postgresql" | "oracle" | "dm" | "kingbase" | "gbase",
      host: "localhost",
      port: 3306,
      database: "mydb",
      user: "root",
      password: "<encrypted>"  // AES 加密
    },
    test: { ... }
  }
}
```

**密码加密**：AES-256-GCM（密钥从机器特征派生）

**用途**：
- 导入读结构（第一版）
- 数据集从库导入数据
- diff A 对账（下版）

---

## 8. 打包

### Electron 打包

```
app/
  resources/
    jre/              # 精简 JRE (按平台)
    db-connector.jar  # Java 连接器 + JDBC 驱动
```

**electron-builder 配置**：
```json
{
  "extraResources": [
    {"from": "resources/jre/${os}-${arch}", "to": "jre"},
    {"from": "resources/db-connector.jar", "to": "db-connector.jar"}
  ]
}
```

**运行时**：
- 开发模式：用系统 Java
- 打包模式：用 resources/jre/bin/java

**安装包体积**（单平台）：~170MB

### 全局配置存储

- 最近项目、用户偏好：`~/.local/share/aqua/`

---

## 9. 第一版功能边界

### 包含

- [x] 表结构建模（字段/索引/分组）+ Vue UI 编辑
- [x] 业务类型系统（内置不可删 + 自定义）
- [x] Enum 系统（全局/内联，特殊业务类型）
- [x] DDL 生成（7 方言，大写，含数据集 INSERT）
- [x] 代码生成（Java 实体 + 前端 JSON + StrConst）
- [x] diff + 生成 ALTER
- [x] 导入（连库读结构，6 种数据库）
- [x] 数据集管理（JSON/SQLite 双格式，库↔数据集↔库迁移）
- [x] CLI（generate 统一命令，plugin 化 type）
- [x] 表/字段复制粘贴（跨项目/跨表）
- [x] Electron 打包（精简 JRE + db-connector.jar）
- [x] 前端全部页面用 JSON 写（json-ui）

### 不含（下版/永不做）

- [ ] diff A（JSON vs 数据库对账）—— 下版
- [ ] 数据库设计文档产出（Markdown）—— 暂不排期
- [ ] 自动执行 DDL —— 永不做
- [ ] 物理外键 —— 不做
- [ ] CSV 导入导出 —— 以后考虑
- [ ] Git 版本集成 —— 下版
- [ ] 多人协作/权限 —— 下版
- [ ] 撤销/重做 —— 不做

---

## 10. 与现有工具的对齐

### 10.1 rainbow-dbaccess

- 实体注解：完全对齐 `@Table/@Id/@GeneratedValue/@Column`
- autoGenerate：对齐 strategy(default/now/自定义) + param + timing
- Enum：对齐 CodeEnum / 普通枚举
- FieldMapper：不在业务类型声明（由 rainbow 运行时自动推导或手动指定）

### 10.2 json-ui

- DataModelSchema：字段命名完全对齐
- 前端 JSON 生成：映射粗粒度 DataType（INT/LONG/DECIMAL/TINYINT→NUMBER）
- bizType meta.json：格式对齐（简化为 string/number）
- 全部 UI 页面用 json-ui 的 JsonRender 渲染

---

## 11. 待定/后续

- 工具最终命名（影响全局配置路径 `~/.local/share/aqua/`）
- 内置业务类型清单（从外置配置文件读，具体清单待定）
- DM/KingBase/GBase 的 DDL 方言细节（开发时按文档适配）
- Oracle 11g 序列+触发器 vs 12c+ IDENTITY 的选择策略
