# 数据集重构

## Goal

数据集从"手动打开/保存文件"重构为"项目目录扫描 + DBeaver 式编辑 + 数据库导入导出"。对齐 design.md §276-282(主文件名.数据集名.{json|db},自动扫描)。

## 背景

当前 DatasetManage:pickDatasetOpen/Save(文件对话框)+ 行数据编辑 + 整体存回。与 design.md 不符(应自动扫描同目录,无手动 open/save)。

design.md:
- 命名:`主文件名.数据集名.{json|db}`(如 myproject.dev.json)
- 打开项目自动扫描同目录匹配文件
- 新建数据集设定格式(json/sqlite)
- 库↔数据集↔库迁移

## 需求

### 1. 文件规则 + 新建
- 文件命名:`{主文件名}.{数据集名}.data`(JSONL 格式,后缀 .data)
- 新建:弹窗录入数据集名(统一 .data 格式,无 json/db 选择),**名字不可变**
- 内容:JSONL 每行 `{"table":"SYS_USER","row":{"ID":1,"NAME":"admin"}}`,不存表结构(结构用主项目)
- 保存时**按主键排序**:每表行按第一个主键字段值排序(数字序/字符串序),无主键保持原序

### 2. 目录扫描 + 下拉
- 进入数据集页,扫描项目目录匹配 `{主文件名}.*.data`
- 下拉选择框列出数据集(显示数据集名)

### 3. 编辑(DBeaver 模式)
- 选数据集 -> 加载行数据到表格(可编辑)
- **数据表表头显示字段中文名**(Field.name,从主项目结构,非 code)
- **dirty 时显示保存/取消按钮;无修改隐藏**
- 保存:写回文件;取消:恢复原始数据

### 4. 数据库导入/导出
- **导入**(数据库 -> 数据集):查表数据写入数据集,**覆盖**数据集原数据
- **导出**(数据集 -> 数据库):**TRUNCATE + INSERT** 覆盖数据库表数据
- 导出前**提醒用户**(覆盖危险)
- 选表(复用 TableSelectDialog,同 DDL 导出)
- 使用数据源(已配连接)

### 5. DDL 导出加数据集 INSERT
- DDL 导出界面增加**数据集下拉框**
- 选中某个数据集时,DDL 输出追加该数据集的 **INSERT 语句**
- 未选则只导出 DDL(不追加 INSERT)

### 6. 底层 INSERT 支持
- connector(Rust native / Java JDBC)支持执行 INSERT/TRUNCATE
- 当前 connector 只读(反解),需扩展写能力
- 数据集放弃 SQLite,纯 JSONL(.data),移除 rusqlite 依赖

## 验收标准

- [ ] 新建数据集:弹窗录名字(统一 .data);文件名 = 主文件名.数据集名.data
- [ ] 数据集页:下拉列目录扫描的 .data 文件
- [ ] 编辑:DBeaver 式,dirty 时保存/取消按钮,无修改隐藏
- [ ] 导入:数据库->数据集,覆盖,选表,用数据源
- [ ] 导出:数据集->数据库,TRUNCATE+INSERT,覆盖提醒,选表,用数据源
- [ ] DDL 导出:加数据集下拉,选中追加 INSERT 语句
- [ ] 底层 INSERT:connector 支持写(native + JDBC)
- [ ] cargo test/clippy + vue-tsc 全过,现有功能不回归

## 非目标

- 数据集格式切换(json<->db,独立功能)
- 数据集行数据类型推断(保持当前字段类型映射)
- 多数据集同时打开

## 待定(进 design)

- connector INSERT 实现:native(MySQL/PG)/ JDBC(connector.jar execute_update)
- TRUNCATE+INSERT SQL 生成
- 数据集 dirty 跟踪:rowsMap 变更检测
- DDL 导出 + INSERT:数据集读取 + INSERT 生成(按选中表)
- 主键排序:按第一个主键字段值(数字/字符串)
