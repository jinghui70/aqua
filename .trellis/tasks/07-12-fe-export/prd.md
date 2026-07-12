# 导出 (DDL/diff/StrConst 弹窗)

## 背景
parent frontend-rework child。菜单"导出>DDL/diff/StrConst"打开弹窗。§6.9。

## 目标
3 个导出弹窗,均支持 预览+复制+下载。

## 包含
### 导出 DDL(§6.9)
- 方言下拉 / 范围(全部/分组/指定表)/ 预览 / 复制 / 下载 .sql

### 导出 diff
- 选旧版 schema.json(输入路径)/ ALTER DDL(选方言)/ 预览 / 下载
- (差异报告文本 本期可选,先做 ALTER DDL)

### 导出 StrConst(§6.9)
- 范围(全部/分组/指定表)/ 包名 / 类名 / 预览 / 复制 / 下载

## 后端补 command
- generate_ddl_command 扩展: 加 tables/group 过滤参数
- generate_strconst_command 新增
- generate_alter_command 新增(读旧 project + diff + alter)

## 验收标准
- [ ] 后端: DDL 加过滤参数, strconst/alter command 新增 + 注册
- [ ] ExportDialog 组件(3 种模式 or 3 个弹窗)
- [ ] useMenuActions export.ddl/diff/strconst -> 打开弹窗
- [ ] 预览 + 复制 + 下载
- [ ] cargo build + pnpm build 通过

## 约束
- 复用 generators::{ddl,strconst}, diff+alter
- useDownload 下载, element-plus el-dialog
