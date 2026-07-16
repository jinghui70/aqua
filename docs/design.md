# 数据库结构管理工具 - 需求文档(最终版)

**版本**: v4.0
**日期**: 2026-07-13
**状态**: 最终需求(含已实现 + 明确待实现)
**技术架构**: 见 [`architecture.md`](./architecture.md)(Tauri 2.x + Rust + Vue3/element-plus + Java connector)
**访谈记录**: [`grill-me-2026-07-11.md`](./grill-me-2026-07-11.md)

> 本文档是**业务需求** source of truth(数据模型/功能/UI/边界)。技术架构以 `architecture.md` 为准。旧版(Electron+Node+Java)技术栈/打包章节已删除,归 architecture.md。

---

## 1. 项目定位

### 1.1 核心目标

**保证数据结构的唯一性,作为系统开发依据。**

- JSON 是 Single Source of Truth (SSOT)
- DDL、代码、前端数据模型均从 JSON 派生
- **这是数据库设计工具,不是管理工具**--产出 DDL/ALTER 文本,永不自动执行

### 1.2 核心能力

1. **表结构建模**:定义表/字段/索引/分组,细粒度逻辑类型
2. **业务类型系统**:跨前后端共享语义层(内置 + 自定义)
3. **Enum 系统**:全局/内联枚举,特殊业务类型
4. **DDL 生成**:7 种方言(MySQL/PG/Oracle/DM/KingBase/GBase/H2)
5. **代码生成**:Java 实体(rainbow-dbaccess 注解)+ 前端 JSON(json-ui 兼容)+ StrConst
6. **diff + ALTER**:JSON 版本对比,生成 ALTER DDL
7. **导入**:连库读结构(MySQL/PG 走 Rust native;Oracle/信创/H2 走 Java JDBC)
8. **数据集管理**:JSON/SQLite 双格式,库↔数据集↔库迁移
9. **Undo/Redo**:主文件任何变动可撤销/重做(待实现)
10. **CLI**:generate(统一命令,内置 4 个 generator)

### 1.3 工作流

**新项目**:
```
定义 JSON -> 生成 DDL 建库 -> 生成代码 -> 定义数据集
```

**存量接管**(一次性反向):
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

## 2. 技术架构

详见 [`architecture.md`](./architecture.md)。要点:

- **Tauri 2.x 桌面** + Rust 后端(`crates/aqua-core` 纯逻辑核心 + `src-tauri` 壳) + Vue3/element-plus 前端(`app/`) + Java connector(复用,`connector/`)
- **连接层混合**:MySQL/PG 走 Rust native 免 Java;Oracle/信创/H2 走 Java JDBC(用户自备 JDK 17+)
- **两类数据库**:内置方言(native,硬编码) + JDBC 方言(外置 JDBC 驱动,零重编译扩展)
- **CLI**:Tauri 二进制双模式(无 args 开 GUI / 有 args 走 CLI)
- **打包**:~20MB(connector.jar 内置,JRE 用户自备,无 Chromium)
- **不做**:自动更新、SSH 隧道、Web/Docker 部署。**仅中文**。

---

## 3. 数据模型

### 3.1 逻辑类型(dataType)

| 类型 | 属性 | 说明 |
|---|---|---|
| VARCHAR | length | 变长字符串 |
| CLOB | - | 长文本 |
| TINYINT | - | 小整数(-128~127) |
| INT | - | 32位整数 |
| LONG | - | 64位整数 |
| DECIMAL | precision, scale | 精确小数 |
| DATE | - | 日期 |
| DATETIME | - | 日期时间 |
| BLOB | - | 二进制 |

**不含**:BOOLEAN(跨库不一致,用 TINYINT/VARCHAR+业务类型)、JSON(用 BLOB/CLOB)、DOUBLE(DECIMAL 足够)

### 3.2 字段模型(Field)

```typescript
interface Field {
  prop: string                // Java/TS 属性名(驼峰,如 userName)
  code: string                // 数据库字段名(大写蛇形,如 USER_NAME)
  name: string                // 中文名(DDL COMMENT)
  dataType: DataType          // 逻辑类型(9 种)
  length?: number             // VARCHAR 长度
  precision?: number          // DECIMAL 精度
  scale?: number              // DECIMAL 小数位
  bizType?: string            // 业务类型 code
  bizTypeData?: unknown       // 业务类型配置(单 field 直接存值,多 field 存对象)
  isKey?: boolean             // 主键
  notNull?: boolean           // 非空
  defaultValue?: string       // DDL DEFAULT 子句
  autoGenerate?: {            // 应用层生成(@GeneratedValue)
    enabled: boolean
    strategy: "default"|"now"|string  // default=雪花, now=当前时间, 自定义
    param?: string
    timing: "INSERT"|"INSERT_UPDATE"
  }
  enum?: string | InlineEnum  // string=引用全局枚举 code,object=内联枚举
  comment?: string            // 详细描述(Java Javadoc 用,不进 DDL;UI 放最后)
}
```

**规则**:
- `prop` 与 `code`:新建字段时从 code 自动派生 prop(蛇形->驼峰),可手动修改
- `enum` 只支持 VARCHAR

### 3.3 表模型(Table)

```typescript
interface Table {
  code: string                // 表名(大写蛇形)
  name: string                // 中文名(DDL COMMENT,即表备注)
  group: string               // 引用分组 code(一表一分组)
  fields: Field[]             // 字段列表
  indexes?: Index[]           // 索引
}

interface Index {
  name?: string               // 索引名(空则自动生成)
  fields: string[]            // 字段 code 列表(有序,支持调整顺序)
  unique: boolean             // 唯一索引
}
```

> **变更**:去掉 `Table.comment`--`name` 即中文名/备注,不再单列备注字段。

### 3.4 业务类型(BizTypeDefine)

```typescript
interface BizTypeDefine {
  bizType: string             // code(如 "Date")
  name: string                // 中文名
  description?: string        // 说明
  supportedDataTypes: Array<{
    dataType: DataType
    defaultLength?: number
    defaultPrecision?: number
    defaultScale?: number
  }>
  bizTypeData?: {
    fields: Array<{
      name: string
      type: "string" | "number"
      description?: string
      required?: boolean
      default?: string | number  // 参数默认值(选 bizType 时初始化 bizTypeData)
    }>
  }
}
```

> **变更**:`bizTypeData.fields[]` 加 `default`(参数默认值)。

**内置业务类型**:
- 外置资源文件 `resources/builtin-biztypes.json`,启动加载
- 项目不可删改,升级产品才变
- 当前含 `Date` 示例(format 参数,default "YYYYMMDD",VARCHAR/8)

**自定义业务类型**:
- 项目独享,可增删改
- 无版本管理,改动自动检查并提示

**用户交互**:
- 先选数据类型 -> bizType 下拉只显示支持该类型的业务类型 -> 自动填充默认值
- 先选业务类型 -> dataType 下拉只显示该类型支持的类型 -> 自动填充默认值
- 选 bizType 时用 `default` 初始化 bizTypeData(单 field 存值,多 field 存对象)

**单 field 简化保存**:
- bizTypeData 只有一个 field 时,字段实例的 bizTypeData 直接保存值(非对象)
- 多 field 时保存对象

### 3.5 Enum(特殊业务类型)

**EnumDefine(全局枚举,schema.json 顶层 enums 数组)**:

```typescript
interface EnumDefine {
  code: string                // 枚举标识(如 "EnumGender")
  name: string                // 中文名(如 "性别")
  package: string             // 相对子路径,拼到 basePackage 下
  hasCode?: boolean           // true=CodeEnum 派生存 code,false/无=普通枚举存 id
  values: EnumValue[]
}

interface EnumValue {
  id: string                  // 枚举项标识("MALE"),Java 枚举名,前端用
  name: string                // 中文显示("男")
  code?: string               // 存储值("M"),hasCode=true 时必填
  color?: string              // 预置颜色(写死代码)
}
```

**color 预置列表**(写死代码):success / error / warning / info / primary / danger / red / orange / yellow / green / blue / purple / grey

**hasCode 行为**:
- `hasCode=true`:数据库存 code,Java 生成 `MALE("M","男") implements CodeEnum`
- `hasCode=false`:数据库存 id,Java 生成普通枚举 `MALE // 男`
- 校验:hasCode=true 时每个 value 必须有 code

**字段引用枚举**(二选一):
- 引用全局:`field.enum = "EnumGender"`(string)
- 内联:`field.enum = {name, hasCode, values}`(object,无 code/package)

**删除级联**:删除全局枚举时,统计引用该 code 的字段(按表聚合提醒),确认后级联清除 `field.enum`;若字段 `bizType==="Enum"` 一并清 `bizType`(避免无 enum 的不一致)。内联枚举不受影响。

**Java 生成**:
- 全局枚举:生成独立 enum 类(自身 package)
- 内联枚举:生成独立 enum 类(共享表的 package)

### 3.6 项目结构

**项目 = 目录**,文件命名用项目主文件名前缀:

```
myproject/                        # 项目目录
  myproject.json                  # 主文件(项目名.json,Project 结构)
  myproject.dev.json              # JSON 数据集(主文件名.数据集名.json,入 Git)
  myproject.test.db               # SQLite 数据集(主文件名.数据集名.db,.gitignore)
  .dbconfig.json                  # 数据源配置(.gitignore)
  .gitignore
```

**.gitignore**:
```
*.db
.dbconfig.json
```

**schema.json 顶层**:

```typescript
interface Project {
  version: string             // 产品版本号
  name?: string               // 项目中文名(可选,旧 schema 兼容;显示用)
  basePackage: string         // 全局根 package(如 "com.example")
  bizTypes: BizTypeDefine[]   // 业务类型(自定义;内置单独加载)
  enums: EnumDefine[]         // 全局枚举
  groups: GroupDefine[]       // 分组(显式定义,数组顺序即排序)
  tables: Table[]             // 表结构
}

interface GroupDefine {
  code: string                // 分组标识(如 "order")
  name: string                // 中文名(如 "订单模块")
}
```

> **变更**:`Project` 加 `name`(项目中文名)。

**分组规则**:
- 单层分组
- 显式定义(groups 数组),无 sort,数组顺序即排序
- 一表一分组(table.group 引用单个 code)
- 分组/表顺序支持拖拽调整,表支持跨分组拖拽

**数据集文件规则**:
- 与主文件同目录
- 命名:`主文件名.数据集名.{json|db}`
- 打开项目时自动扫描同目录,匹配命名规则的数据集,无则空
- 新建数据集时设定格式(json/sqlite),切换格式是独立功能点

---

## 4. 核心功能

### 4.1 DDL 生成

**输入**:Project + 目标方言(MySQL/PG/Oracle/DM/KingBase/GBase/H2)
**输出**:建表 SQL 文本(CREATE TABLE + INDEX + 可选数据集 INSERT)
**规则**:表名/字段名/关键字**大写**,COMMENT 用 name 字段

#### 逻辑类型 -> 物理类型映射

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

autoGenerate 是应用层生成(@GeneratedValue),DDL 不体现。

#### 数据集 INSERT

- 指定数据集时附加 INSERT(标准 SQL,不分方言)
- 表内行按主键排序,表间按数据集数组顺序

### 4.2 代码生成

#### 4.2.1 Java 实体(按 rainbow-dbaccess 规范)

**package 规则**:`{basePackage}.{groupCode}.{entity}`,各段可编辑

**生成规则**:
- 类名:表 code 派生 PascalCase(如 `USER_INFO` -> `UserInfo`),可编辑
- 属性名:field.prop(驼峰)
- 类型映射:VARCHAR/CLOB->String,TINYINT/INT->Integer,LONG->Long,DECIMAL->BigDecimal,DATE->LocalDate,DATETIME->LocalDateTime,BLOB->byte[]
- 注解:`isKey=true` -> `@Id`;`autoGenerate` -> `@GeneratedValue(strategy, param, timing)`;非标准命名 -> `@Column(name)`
- `comment` -> Javadoc(可开关)
- 枚举字段:生成对应 enum 类(全局/内联)

**配置项**(表编辑页 Java Tab):包名 / 类名 / Lombok @Data 开关 / 生成注释开关

#### 4.2.2 前端 JSON(按 json-ui 规范)

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

> 前端 JSON 服务于外部 json-ui 项目(非 aqua 自身 UI)。

#### 4.2.3 StrConst(字符串常量类)

```java
public class StrConst {
    // 表名
    public static final String SYS_USER = "SYS_USER";
    // 字段名
    public static final String USER_ID = "USER_ID";
}
```

**规则**:表名 + 字段名都导出;跨表重复字段名去重;范围可选(表/分组/全部);package `{basePackage}.const`(可编辑);类名默认 `StrConst`(可编辑)。

### 4.3 diff + ALTER

**diff 输入**:两份 Project JSON(当前 vs 旧版)
**diff 输出**:结构化差异列表(DiffResult)

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

**ALTER 生成**(从 DiffResult,按方言):

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

**产物**:可下载 .sql 文件,分步骤注释(便于 DBA 分段执行)

### 4.4 导入(连库读结构)

**流程**:连库 -> Driver 读 JDBC 原始元数据(物理类型+length/precision/scale)-> 反解逻辑类型 -> Project JSON

**反解分库归属**:
- Native 库(MySQL/PG):反解写死 Rust 各驱动模块
- Java 库(Oracle/信创/H2):反解在 Java 侧 Dialect 子类;外置 JDBC 驱动 jar 经 URLClassLoader 加载(databases.json 记录 installed)

**不猜 bizType**(留空,人工标注)

**导入表结构只导结构,不导数据**

### 4.5 数据集

#### 格式

- **JSON 数据集**(`.json`):可读,入 Git,小数据量
- **SQLite 数据集**(`.db`):二进制,不入 Git,大数据量
- 格式从扩展名判断,创建时选

#### SQLite 存储规则(保持精度)

| 逻辑类型 | SQLite 存储 |
|---|---|
| VARCHAR/CLOB | TEXT |
| TINYINT/INT/LONG | INTEGER |
| DECIMAL | TEXT(字符串,避免精度丢失) |
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

- key 用数据库字段名(与表结构 code 一致)
- DECIMAL 用字符串,INT/LONG/TINYINT 用数字
- DATE/DATETIME/VARCHAR/CLOB 用字符串
- BLOB 用 base64
- 空值用 null(不省略 key)

#### 数据集必须匹配项目表结构

- 数据集的表 = 项目 schema.json 的表
- 不支持自由数据

#### 导入(库 -> 数据集)

| 模式 | 结构要求 | 映射 |
|---|---|---|
| 单表导入 | 异构可映射 | 支持字段映射(源表与目标表可不同名) |
| 多表/全表导入 | 必须结构一致 | 不映射,直接导,报错提示 |

- **导入只有覆盖,没有追加**

#### 迁移路径

库A -> 导入到数据集 -> 导出 INSERT -> 库B 执行(数据集是中间载体)

---

## 5. 命令行(CLI)

### 5.1 定位

Tauri 二进制双模式:无 args 开 GUI,有 args(`aqua generate ...`)走 CLI 不开窗。generate 逻辑在 aqua-core。

### 5.2 命令

```bash
aqua generate --project ./myproject --type <type> [options]
# 输出到 stdout,用操作系统管道重定向
```

**type 种类**(内置 4 个,扩展靠改程序重编译):

#### type: ddl
```bash
aqua generate --project ./myproject --type ddl \
  [--dialect mysql]              # 默认 mysql
  [--table user,order]           # 可选,逗号分隔
  [--group core]                 # 可选,与 --table 互斥
  [--dataset test]               # 可选,包含数据集(仅 JSON 数据集)
```

**方言**:mysql / postgresql / oracle / dm / kingbase / gbase / h2
**--dataset 限制**:只支持 JSON 数据集(SQLite 二进制无法流式输出 stdout)

#### type: java(单表)
```bash
aqua generate --project ./myproject --type java \
  --table user                   # 必需,只支持单表
  [--package com.example.entity] # 可选,默认 basePackage.groupCode.entity
```

#### type: json(单表)
```bash
aqua generate --project ./myproject --type json --table user
```

#### type: strconst
```bash
aqua generate --project ./myproject --type strconst \
  [--table user,order]           # 可选,默认全部
  [--group core]                 # 可选
  [--package com.example.const]  # 可选,默认 basePackage.const
  [--classname StrConst]         # 可选,默认 StrConst
```

---

## 6. UI

### 6.1 菜单栏

```
[文件]  [配置]  [导出]  [帮助]

文件: 新建项目 / 打开项目 / 保存 / 另存为 / 最近项目 / 退出
配置: 项目设置 / 业务类型管理 / 枚举管理 / 数据集管理 / 数据源配置
导出: DDL / diff / StrConst
帮助: 用户指南 / 关于
```

- 原生窗口菜单(Tauri MenuBuilder),菜单事件 emit 到前端分发
- **Undo/Redo**:编辑菜单或文件菜单提供撤销/重做(待实现,见 §6.12)

### 6.2 主界面

```
[菜单栏]
[工作区]
  ┌─────────────┬──────────────────────────────┐
  │  分组树     │   表编辑区                    │
  │ (侧边栏)   │   [表 code/中文名 编辑]       │
  │ 📁 订单模块 │   [字段][索引][DDL][Java][JSON][数据]
  │    ├ USER   │                              │
  │    └ ROLE   │   Tab 内容...                 │
  │  [+ 分组]   │                              │
  └─────────────┴──────────────────────────────┘
[状态栏]
```

**分组树**:
- `select-none`,支持拖拽:分组排序、表跨分组移动、表组内排序
- 分组节点只显示中文名;表节点显示 `code (中文名)`
- 新建分组/表用正经对话框(分组填 code+中文名;表填 code+中文名)
- hover 显示操作:分组(+表/改/删)、表(改/删/复制)
- 表复制:复制表结构为新表(code 加后缀)
- 支持搜索

**表/字段复制粘贴**:Ctrl+C/V,code 加后缀,支持跨项目/跨表(待实现)

### 6.3 表编辑页(6 个 Tab)

表编辑区顶部:表 code 编辑(全局唯一校验 + 页签标题/路由同步更新)+ 中文名编辑。**去掉"改分组"**(树拖拽改)。

#### Tab: 字段
- 字段表格(列:拖拽手柄/#/code/prop/名称/类型/业务类型/主键/非空/自动生成/默认值/备注/操作)
- 行内编辑(code/prop/名称/类型/默认值/备注)+ 详情弹窗(autoGenerate/bizTypeData/enum)
- code↔prop 联动(蛇形->驼峰)
- 类型↔bizType 双向联动 + 默认值填充
- bizType=Enum -> 特殊配置(引用/内联);其他 bizType -> fields 配置(含 default 初始化)
- 拖拽排序(forceFallback,select-none)
- **详情弹窗加宽,一屏显示所有内容不滚动;备注放最后**

#### Tab: 索引
- 索引表格(name/fields/unique)
- **字段选择规范化**(非简单下拉)+ **索引内字段顺序可调**
- 新增/删除

#### Tab: DDL
- 选方言(mysql/postgresql/oracle/h2)实时预览单表 DDL
- 复制 / 下载

#### Tab: Java(只读预览 + 下载)
- 配置项:包名 / 类名 / Lombok / 注释开关
- 配置改动 -> 预览实时刷新
- 复制 / 下载

#### Tab: JSON(只读预览 + 下载)
- 无配置项,直接输出 json-ui 兼容格式
- 复制 / 下载

#### Tab: 数据(新)
- 数据集下拉(同目录自动扫描的数据集)+ 刷新 + 保存
- 该表在该数据集的数据网格(行编辑)
- 新建/删除数据集、切换格式、复制数据集在配置菜单"数据集管理"弹窗

**所有 Tab 内部**:margin-top + 占满空间 + 代码区内部滚动(不撑开页面)。

### 6.4 数据集

**表编辑页"数据"Tab**(见 §6.3):选数据集看该表数据,行编辑。

**配置菜单"数据集管理"弹窗**:
- 数据集列表(同目录扫描)
- 新建(设定 json/sqlite 格式)
- 删除
- 切换格式(独立功能点)
- 复制数据集

**文件规则**(见 §3.6):同目录,命名 `主文件名.数据集名.{json|db}`,打开项目自动扫描。

> **变更**:删掉原独立"数据集管理页"(两棵树),改为表数据 Tab + 管理弹窗。

### 6.5 业务类型管理(配置菜单)

- 左列表:内置(只读,内置 tag,无删/编辑)+ 自定义(可增删改)
- 右编辑:bizType(code 只读)/名称/描述/supportedDataTypes 子表/bizTypeData.fields 子表(参数名/类型/默认值/描述/必填)
- 内置条目表单 disabled + 顶部提示"内置业务类型只读"
- 新建自定义 bizType 重名校验含内置

### 6.6 Enum 管理(配置菜单)

- 全局枚举列表 + 编辑(code/name/package/hasCode/values[id/name/code?/color?])
- 删除级联:统计引用字段(按表聚合提醒),确认后清 field.enum + 若 bizType=Enum 清 bizType

### 6.7 数据源配置(配置菜单)

- 数据源列表(dev/test/prod 等)
- 新增/编辑/删除(增删改后自动落盘)
- 表单:类型/主机/端口/数据库/用户/密码
- 测试连接
- 存 `.dbconfig.json`(密码 AES-256-GCM 加密)
- 打开项目时加载,无项目路径时仅内存

### 6.8 导入(文件菜单/导入)

4 步向导:
1. 选数据源(dev/test/prod/新建)
2. 选表(多选 + 全选/反选/搜索)
3. 导入选项(分组选择)
4. 确认导入(同名表提示:覆盖/跳过)

**导入表结构只导结构,不导数据**
**入口要明显暴露**(用户反馈"没看到")

### 6.9 导出(导出菜单,3 项)

所有导出支持**预览 + 复制 + 下载**。

#### 导出 DDL
```
目标方言: [MySQL ▼]
范围: ○ 全部表  ○ 按分组  ○ 指定表(弹树选择)
包含数据集: [无 ▼](可选,JSON 数据集)
☐ 仅表结构
[预览]  [下载 .sql]
```

#### 导出 diff
```
对比文件: [选择旧版 schema.json]
输出:
  ○ 差异报告(文本/HTML/Markdown)
  ○ ALTER DDL(选方言)
[生成]  [下载]
```

#### 导出 StrConst
```
范围: ○ 全部表  ○ 按分组  ○ 指定表(弹树选择)
包名: [com.example.const]
类名: [StrConst]
[预览]  [下载 StrConst.java]
```

> **变更**:导出选表改弹树选择;DDL/StrConst 导出可勾选 JSON 数据集一起导出。

### 6.10 欢迎页 / 新建项目

**欢迎页**:
- 新建项目 / 打开项目
- 最近项目列表(存用户数据目录):**路径之上显示项目中文名**
- 无中文名时 fallback basePackage

**新建项目对话框**:输入项目名(中文)+ basePackage(英文包名),确认后创建。

### 6.11 横切规范

- **标签中文化**:UI 标签统一中文,不英文混用。如 `enum` -> 枚举、`basePackage` -> 根包名、`code`/`prop`/`bizType`/`hasCode` 等改中文标签。技术术语 DDL/Java/JSON 保留。
- **页签布局**:所有 Tab 内部 margin-top + 占满空间 + 代码区内部滚动。
- **关闭行为**:关闭项目回退欢迎页。**改动未保存时,关闭项目/打开新项目/退出应用提醒保存**。
- **select-none**:树/表格等交互区禁用文本选中。
- **弹窗**:禁用 window.prompt/confirm/alert,用 ElMessageBox/ElMessage。
- **样式**:unocss 原子类,presetRemToPx(baseFontSize:4),尺寸用 px,不写 scoped CSS。
- **包管理**:pnpm(禁用 npm)。

### 6.12 Undo/Redo(待实现)

**需求**:主文件(Project)的任何变动支持撤销/重做。

**范围**:
- 表/字段/索引的增删改
- 业务类型/枚举/分组的增删改
- 项目设置(name/basePackage)改动
- 字段拖拽排序、树拖拽

**UI**:
- Ctrl+Z 撤销,Ctrl+Shift+Z(或 Ctrl+Y)重做
- 菜单撤销/重做项

**边界**:
- 仅当前会话内(不跨会话)
- 保存后是否清空历史:待定(实现时定)

**实现**:难度大,单独建任务做 design 规划。候选方案:
- 命令模式(每个变动封装 Command,do/undo)--精细但工作量大
- 状态快照(每次变动存 Project 深拷贝,undo 回退)--简单但内存/性能
- 不可变数据 + 历史栈

需在 design 阶段权衡:粒度(每次按键 vs 每次操作)、内存、与 Pinia 响应式集成、性能(深拷贝频率)。

---

## 7. 数据源配置

**存储**:`.dbconfig.json`(项目目录,进 .gitignore)

```typescript
{
  sources: {
    dev: {
      dialect: "mysql" | "postgresql" | "oracle" | "dm" | "kingbase" | "gbase" | "h2",
      host: "localhost",
      port: 3306,
      database: "mydb",
      user: "root",
      password: "<encrypted>"  // AES-256-GCM 加密
    },
    test: { ... }
  }
}
```

**密码加密**:AES-256-GCM
- 密钥为 32 字节随机值,存用户数据目录 `key`(权限 600)
- 首次运行生成,后续复用
- 密文格式 base64(nonce ‖ ciphertext+tag)
- 空密码不加密

> **变更**:密钥策略从"机器特征派生"改为"用户数据目录随机密钥"(更稳可控;机器特征重装/换硬件会变导致无法解密)。

**用途**:
- 导入读结构
- 数据集从库导入数据
- diff A 对账(下版)

---

## 8. 功能边界

### 已实现

- [x] 表结构建模(字段/索引/分组)+ Vue UI 编辑
- [x] 业务类型系统(内置只读 + 自定义 + 参数默认值)
- [x] Enum 系统(全局/内联 + 删除级联)
- [x] DDL 生成(7 方言,大写,含数据集 INSERT)
- [x] 代码生成(Java 实体 + 前端 JSON + StrConst)
- [x] diff + 生成 ALTER
- [x] 导入(连库读结构,native + JDBC 混合)
- [x] 数据集管理(JSON/SQLite 双格式,核心 CRUD)
- [x] 数据源配置持久化(.dbconfig.json + AES-256-GCM)
- [x] 项目中文名 + 项目设置对话框
- [x] CLI(Tauri 双模式,generate 内置 4 generator)
- [x] 表删除级联提醒(按表聚合)

### 待实现(本批交互优化 + Undo/Redo)

- [ ] 欢迎页最近项目显中文名 + 新建项目对话框
- [ ] 分组树 select-none + 拖拽(分组排序/表跨分组/组内)
- [ ] 新建分组/表正经对话框;节点显示(分组只中文名,表 code+中文名);表复制
- [ ] 表编辑页:去改分组、表 code 编辑(唯一校验+页签同步)、字段弹窗加宽、字段备注后置
- [ ] 索引编辑规范化 + 字段顺序可调
- [ ] 表编辑页加"数据"Tab(选数据集看该表数据)
- [ ] 数据集重构(表数据 Tab + 管理弹窗 + 同目录文件规则 + 删旧页)
- [ ] 标签统一中文化
- [ ] 页签布局(margin-top/占满/内部滚动)
- [ ] 导入入口暴露
- [ ] 导出选表树选择 + 可勾选 JSON 数据集
- [ ] 关闭项目回欢迎页 + 未保存提醒
- [ ] Undo/Redo(主文件变动历史,单独规划)

### 不含(下版/永不做)

- [ ] diff A(JSON vs 数据库对账)-- 下版
- [ ] 数据库设计文档产出(Markdown)-- 暂不排期
- [ ] 自动执行 DDL -- 永不做
- [ ] 物理外键 -- 不做
- [ ] CSV 导入导出 -- 以后考虑
- [ ] Git 版本集成 -- 下版
- [ ] 多人协作/权限 -- 下版
- [ ] 自动更新 -- 不做
- [ ] SSH 隧道 -- 不做(直连)
- [ ] Web/Docker 部署 -- 不做(纯桌面)

---

## 9. 与现有工具的对齐

### 9.1 rainbow-dbaccess

- 实体注解:完全对齐 `@Table/@Id/@GeneratedValue/@Column`
- autoGenerate:对齐 strategy(default/now/自定义) + param + timing
- Enum:对齐 CodeEnum / 普通枚举
- FieldMapper:不在业务类型声明(由 rainbow 运行时自动推导或手动指定)

### 9.2 json-ui

- DataModelSchema:字段命名完全对齐
- 前端 JSON 生成:映射粗粒度 DataType(INT/LONG/DECIMAL/TINYINT->NUMBER)
- bizType meta.json:格式对齐(简化为 string/number)
- 前端 JSON 服务于外部 json-ui 项目(aqua 自身 UI 用 element-plus,不用 json-ui)

---

## 10. 待定/后续

- Undo/Redo 实现方案(命令模式 vs 快照,design 阶段定)
- Undo/Redo 保存后是否清空历史
- 内置业务类型清单(当前 Date 示例,后续扩充)
- DM/KingBase/GBase 的 DDL 方言细节(开发时按文档适配)
- Oracle 11g 序列+触发器 vs 12c+ IDENTITY 的选择策略
- 数据集切换格式功能点
- 表/字段复制粘贴(跨项目/跨表)
