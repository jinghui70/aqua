---
name: aqua
description: 读取项目 aqua 数据表结构(表组/表/字段),按 dba 规范生成 entity Java、按 json-ui 规范生成 DataModel。开发功能、需要了解或落地数据表时使用。
---

# aqua — 数据表结构工具

aqua 管理项目所有数据表结构(前后端字段契约的**单源**)。表定义存于项目里的 `.aqua` 文件(JSON)。本 skill 是 `aqua-cli` 命令行的用法手册——用它**按需**读表结构(不必把整份 `.aqua` 读进上下文)、生成前后端代码。

> `.aqua` 文件在项目中的**存放位置由本项目自己的规范决定**(见项目 spec),本 skill 不假定路径:所有命令的 `<file.aqua>` 由你按项目规范填入。

## 调用方式

本 skill 的二进制在 `bin/` 下,按你的平台选一个,下文命令用 `aqua-cli` 代指:
- **mac**:`./bin/aqua-cli-mac-arm64`(Apple Silicon)/ `./bin/aqua-cli-mac-x64`(Intel)
- **windows**:`.\bin\aqua-cli-win-x64.exe`

## schema 速览(够读懂 `show` 输出)

- **逻辑类型**(10 种):`VARCHAR` `CLOB` `TINYINT` `INT` `LONG` `DECIMAL` `DOUBLE` `DATE` `DATETIME` `BLOB`
- **bizType**:业务类型(如 `Bool` 表示布尔,底层 TINYINT),决定前后端语义与校验
- **分组(group)**:通常对应业务模块;**表**有 `code`(如 `SYS_USER`)与中文 `name`;**字段**有 `code`/`prop`/`name`/逻辑类型/长度/主键/notNull/bizType

## 命令

所有命令末尾传 `.aqua` 文件路径。查询类输出可读文本,`gen` 类把产物打到 **stdout**(你再落到项目规定的位置)。

```
aqua-cli <file.aqua> groups                    # 列所有表组:code  name
aqua-cli <file.aqua> tables [--group <code>]   # 列表(可按组过滤):code  name
aqua-cli <file.aqua> show <table>              # 单表结构(JSON:字段 + 索引)
aqua-cli <file.aqua> gen entity <table> [--package <包名>] [--class-name <类名>]  # → stdout:dba 规范 entity Java
aqua-cli <file.aqua> gen datamodel <table>     # → stdout:json-ui DataModel JSON
```

`<table>` 用表 code(如 `SYS_USER`)。`gen entity`:`--package` 不传则不生成 package 声明(你自行补),传则用全路径包名(如 `cn.com.yusys.frs.sys.user.entity`);`--class-name` 不传则默认 PascalCase(table.code)。

## 典型用法

- **不知项目有哪些模块** → `aqua-cli <file.aqua> groups`
- **看某模块下有哪些表** → `aqua-cli <file.aqua> tables --group sys`
- **写业务代码要了解某表字段** → `aqua-cli <file.aqua> show SYS_USER`(JSON 输出,比读整份 .aqua 省 token)
- **要 entity / DataModel** → `aqua-cli <file.aqua> gen entity SYS_USER` / `aqua-cli <file.aqua> gen datamodel SYS_USER`,把 stdout 产物写到项目规范的位置(entity 进对应功能包、DataModel 进前端页面目录)

## 说明

- 命令**只读**:不会修改 `.aqua`。新增/修改表当前在 aqua 桌面应用里做。
- 出错(表不存在、文件缺失、schema 非法)会打印到 stderr 并非零退出。
- 何时该建表、表放哪个模块等**流程**问题,遵循本项目自己的开发规范,本 skill 只负责"怎么用 aqua 工具"。
