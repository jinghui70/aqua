# 执行计划:项目/树/新建/关闭交互

## 顺序

1. **未保存提醒基础**(project store)
   - 加 dirty ref + watch deep + confirmIfDirty。
   - newProject/openProject/closeProject 接入 confirmIfDirty。
   - saveProject 清 dirty。

2. **关闭项目**
   - store.closeProject()(清状态+数据源+路由)。
   - 菜单加 file.close,useMenuActions 分发。

3. **新建项目对话框**
   - NewProjectDialog.vue(项目名+basePackage)。
   - newProject 改签名(name, basePackage)。
   - 菜单 file.new 打开对话框(替代直接创建)。

4. **欢迎页最近项目中文名**
   - useRecentProjects 改存 {path, name?}。
   - record 接收 name;openProject 传 name。
   - 欢迎页显示 name ?? 文件名。

5. **树 select-none + 拖拽**
   - el-tree draggable + allow-drop(层级约束)+ onDrop(更新 store)。
   - store 加 moveTable / reorderGroups。
   - 树容器 select-none。

6. **新建分组/表对话框**
   - GroupDialog.vue / TableDialog.vue(code+中文名)。
   - 替换 GroupTreeAside 的 prompt 调用。

7. **节点显示**
   - treeData:分组 label=name,表 label=`code (name)`。

8. **表复制**
   - store.duplicateTable(code)。
   - 树表节点 hover 加"复制"操作。

9. **验证**
   - pnpm build + vue-tsc 通过。
   - 人工:新建/拖拽/复制/关闭/未保存提醒各走一遍。

## 注意

- el-tree allow-drop 需覆盖所有拖拽组合,边界测全。
- dirty watch deep 性能留意;Undo/Redo 任务会重构。
- useRecentProjects 改结构注意兼容旧 recent.json(无 name 字段)。
