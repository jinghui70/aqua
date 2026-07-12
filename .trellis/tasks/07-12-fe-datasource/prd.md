# 数据源配置 (弹窗)

## 背景
parent frontend-rework child。菜单"配置>数据源配置"打开弹窗。§6.7。导入向导依赖数据源。

## 目标
数据源列表 + CRUD 弹窗,测试连接。

## 包含
- 数据源列表(名称/type/host/port/database)
- 新增/编辑/删除
- 表单: name/dialect/host/port/user/password/database
- 测试连接(复用 test_connection_command)
- 存 Pinia datasource store(内存态)

## 范围(不含)
- 持久化到 .dbconfig.json + AES 加密(后端单独任务 datasource-persist)
- 本期内存态: 重启丢失,先让导入链路可用

## 验收标准
- [ ] stores/datasource.ts: 数据源列表 + CRUD
- [ ] DataSourceDialog.vue: 列表 + 表单 + 测试连接
- [ ] useMenuActions config.datasource -> 打开
- [ ] AppLayout 挂载
- [ ] 测试连接调 test_connection_command
- [ ] pnpm build 通过

## 约束
- element-plus el-dialog, ElMessageBox, unocss px
- 密码 type=password
