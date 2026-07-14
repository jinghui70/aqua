# 技术设计:项目/树/新建/关闭交互

## 欢迎页最近项目中文名

`useRecentProjects` 当前存路径数组。改为存 `{ path, name? }[]`:
- `record(path, name?)`:打开项目时调,name = currentProject.name ?? basePackage ?? 文件名
- 欢迎页显示 `name ?? 文件名`,路径在下
- 旧记录(只有 path)兼容:无 name 时 fallback 文件名

## 新建项目对话框

新建 `NewProjectDialog.vue`:
- 字段:项目中文名(input)、basePackage(input,默认 com.example)
- 确认 -> `store.newProject(name, basePackage)`
- `project.ts` 的 `newProject` 改签名接收 name+basePackage
- 菜单"文件->新建项目"打开此对话框(替代直接 newProject)

## 树 select-none + 拖拽

`GroupTreeAside.vue` 的 el-tree:
- 加 `draggable` + `select-none` class
- `allow-drop(draggingNode, dropNode, type)`:控制层级
  - 分组节点:只允许 drop 到根(type=prev/next,dropNode 为分组)
  - 表节点:允许 drop 到分组内(type=inner,dropNode 为分组)或分组间 prev/next(同分组内排序)
- `onDrop(draggingNode, dropNode, dropType)`:拖拽完成后更新 store
  - 表跨分组:`table.group = newGroupCode`
  - 分组排序:重排 `project.groups`
  - 表组内排序:重排 `project.tables`(同分组内顺序)
- store 加 `moveTable(tableCode, toGroup, toIndex)`、`reorderGroups(fromIdx, toIdx)`

## 新建分组/表对话框

新建 `GroupDialog.vue` / `TableDialog.vue`(或合并 `PromptDialog`):
- GroupDialog:code + 中文名两字段
- TableDialog:code + 中文名两字段
- 替代当前 `ElMessageBox.prompt` 的 `code|名称` 格式
- store.addGroup/addTable 签名调整(已接收 code+name,OK)

## 节点显示

`treeData` computed:
- 分组 label:`g.name`(去 code)
- 表 label:`${t.code} (${t.name})`

## 表复制

store 加 `duplicateTable(tableCode)`:
- 新 code:`${code}_COPY`,重复则 `_COPY2`、`_COPY3`...
- 新 name:`${name}(副本)`
- 字段深拷贝(结构化克隆),索引深拷贝
- group 同原表
- 加到 project.tables,打开编辑

## 关闭项目回欢迎页

store 加 `closeProject()`:
- currentProject=null, currentPath="", openedTabs=[], activeTab=""
- 数据源清空(datasource.load(""))
- 路由 push "/"
- 菜单"文件"加"关闭项目"项(config.file.close)

未保存时先提醒(见下)。

## 未保存提醒

project store 加 `dirty` ref:
- `watch(currentProject, () => dirty=true, { deep: true })` -- 任何变动标记 dirty
- `saveProject` 成功后 `dirty=false`
- `newProject`/`openProject`/`closeProject` 前检查 dirty:
  ```ts
  async function confirmIfDirty(): Promise<boolean> {
    if (!dirty.value) return true;
    const action = await ElMessageBox.confirm("有未保存改动,是否保存?", "提示", {
      distinguishCancelAndClose: true,
      confirmButtonText: "保存", cancelButtonText: "不保存",
    }).then(() => "save").catch((a) => a === "cancel" ? "discard" : "cancel");
    if (action === "save") { await saveProject(); return true; }
    if (action === "discard") return true;
    return false; // cancel
  }
  ```
- openProject/closeProject 前调 confirmIfDirty,false 则中止

> dirty 用 watch deep 简单实现;Undo/Redo 任务引入历史栈后,dirty 改为"当前态 != 最后保存态"。

## 菜单改动

`lib.rs` 文件菜单加 "file.close" 关闭项目项(保存与另存为之后)。

## 风险

- el-tree 拖拽的 allow-drop 逻辑复杂,需仔细处理 type(prev/inner/next)与节点类型组合。
- watch deep currentProject 性能:项目大时 deep watch 开销。Undo/Redo 任务改历史栈后可去掉 watch。
- 表复制 code 后缀去重需检查全局。
