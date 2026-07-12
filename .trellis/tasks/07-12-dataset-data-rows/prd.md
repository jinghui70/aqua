# 数据集数据行读写

## Goal

打通数据集的数据行编辑闭环:后端提供数据集文件读写(JSON/SQLite 双格式),前端 DatasetManage 可加载、编辑数据行、保存。本任务聚焦核心 CRUD + 文件持久化;"从库导入到数据集""导出 INSERT"拆到后续任务。

## Requirements

- 数据集文件双格式:`.json`(可读,入 Git)与 `.db`(SQLite,大数据量),按扩展名分派。
- 数据集必须匹配项目表结构(§4.5),不支持自由数据;行的 key 用字段 code。
- 类型序列化规则(§4.5):DECIMAL→字符串,INT/LONG/TINYINT→数字,DATE/DATETIME/VARCHAR/CLOB→字符串,BLOB→base64,空值→null(不省略 key)。
- 后端无状态:`dataset_load(path, project)` 读全部行为 JSON;`dataset_save(path, project, data)` 整体写回。行 CRUD 在前端内存进行(与 project open/save 模式一致)。
- 前端 DatasetManage:
  - 新建/打开数据集(OS 文件对话框,选格式)、保存数据集。
  - 表树按分组展示,节点显示真实行数。
  - 数据网格:新增行 / 编辑单元格 / 删除行 / 清空表。
  - 保存前校验数据集表结构与项目一致。

## Acceptance Criteria

- [ ] 打开一个 `.json` 数据集,行数据正确显示在网格,类型正确(DECIMAL 显示为字符串值)。
- [ ] 新增/编辑/删除行后保存到 `.json`,重新打开数据一致。
- [ ] `.db`(SQLite)格式加载/保存往返数据一致,DECIMAL 精度不丢。
- [ ] 表树节点行数随数据变化实时更新。
- [ ] 数据集与项目表结构不一致时,加载/保存给出明确错误。
- [ ] `cargo test -p aqua-core` 通过(新增 dataset 行读写测试)。

## Notes

- 后续任务:从库导入到数据集、导出 INSERT(依赖 driver/ddl)。
- 多数据集下拉列表(扫描项目目录)可留后续;本期以"打开/新建单个数据集文件"为主。
