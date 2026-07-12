# 导入向导 (4步弹窗)

## 背景
parent frontend-rework child。菜单"文件"或独立入口打开。§6.8。依赖 fe-datasource(选数据源)。

## 目标
4 步导入向导: 选数据源 -> 连库列表选表 -> 导入选项 -> 确认导入。导入表结构合并进当前项目。

## 包含(§6.8)
- Step1: 选数据源(datasource store 列表,或临时填)
- Step2: 选表(连库 list_tables,多选/全选/搜索)
- Step3: 导入选项(目标分组选择)
- Step4: 确认(同名表提示覆盖/跳过)
- 导入: import_from_db 得完整 Project,按选中表过滤,合并进 store.currentProject

## 后端
- 已有 import_from_db_command(整库)
- 需要 list_tables 单独 command(Step2 选表用)-> 补 list_tables_command

## 范围(不含)
- 只导结构不导数据(§6.8 明确)
- 异构字段映射(数据集导入才有,§6.4)

## 验收标准
- [ ] 后端 list_tables_command(config -> 表名列表)
- [ ] ImportWizard.vue: el-steps 4 步
- [ ] Step1 选数据源, Step2 连库选表(多选/全选/搜索), Step3 选分组, Step4 确认
- [ ] 导入过滤选中表, 同名提示覆盖/跳过, 合并进 currentProject
- [ ] useMenuActions/入口 打开
- [ ] pnpm build 通过

## 约束
- 无当前项目时先新建, element-plus el-dialog + el-steps
