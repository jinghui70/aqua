# 表编辑多标签页 + 4-Tab

## 背景
parent frontend-rework 核心 child。分组树点表打开的即此编辑器。fe-arch 已建 TableEditor 占位 + 多标签框架。

## 目标
实现表编辑器 4 Tab(§6.3): fields/index/java/json,多表同时打开(标签 keep-alive)。

## 包含
### 表头
- 表 code(只读)/ name / group(下拉选分组) / comment 编辑

### Tab: fields
- 字段表格: prop/code/name/dataType/length(precision,scale)/bizType/isKey/notNull/comment
- 行内编辑(el-table + 内嵌 input/select)
- 新增/删除字段,上移/下移(排序)
- dataType=VARCHAR 显 length;DECIMAL 显 precision/scale
- enum 字段(bizType 相关)本期简化: 显示但不做 enum 特殊配置页(留 fe-enum 联动后续)

### Tab: index
- 索引表格: name/fields(多选表内字段)/unique
- 新增/删除

### Tab: java(只读预览+下载)
- 配置: 包名/类名/Lombok/注释开关
- 调 generate_java_command 实时预览
- 复制 / 下载(Tauri fs 写文件 或浏览器下载)

### Tab: json(只读预览)
- 无配置,后端暂无 frontend_json command -> 本期调用或占位
- 复制 / 下载

## 范围(不含)
- 拖拽排序(用上移/下移替代)
- 复制粘贴(后续)
- enum 特殊配置页(fe-enum 后)
- bizType 表单联动(fe-biztype 后)

## 验收标准
- [ ] TableEditor.vue: 表头 + el-tabs 4 Tab
- [ ] fields Tab: 行内编辑 + 增删 + 排序
- [ ] index Tab: 索引增删,字段多选
- [ ] java Tab: 配置 + 实时预览 + 复制/下载
- [ ] json Tab: 预览 + 复制/下载
- [ ] 多表打开互不干扰(keep-alive,组件 name 匹配)
- [ ] 改动写回 store.currentProject(响应式)
- [ ] pnpm build 通过

## 约束
- 直接改 store 里的 table 引用(Pinia 响应式)
- 复用 useTauri: generateJava(已有)
- frontend_json: 后端有 generators::frontend_json 但无 Tauri command,需补 command 或前端本地转换
- keep-alive: 组件 defineOptions name,解决 fe-arch 遗留
- unocss px + element-plus
