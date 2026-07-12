# fe-arch 技术设计

## 目录结构
```
app/src/
  main.ts              # 挂 router + pinia + element-plus + unocss
  App.vue              # 布局容器(RouterView)
  router/index.ts      # 路由表
  stores/
    project.ts         # 项目状态(currentProject/path/openedTabs)
    ui.ts              # UI 状态(弹窗开关等,按需)
  layout/
    AppLayout.vue      # aside + workspace + statusbar(无 MenuBar,菜单原生)
    GroupTreeAside.vue # 左侧分组树容器(骨架占位,fe-group-tree 填)
    TabWorkspace.vue   # 多标签工作区(el-tabs + router-view + keep-alive)
    StatusBar.vue      # 状态栏
  composables/
    useTauri.ts        # 保留
    useMenuActions.ts  # 监听原生菜单事件 -> 路由/弹窗/store
  views/               # 路由页(各 child 填)
    TableEditor.vue    # 占位(fe-table-editor)
    BizTypeManage.vue  # 占位(fe-biztype)
    EnumManage.vue     # 占位(fe-enum)
    DatasetManage.vue  # 占位(fe-dataset)
  composables/useTauri.ts  # 保留
  types/schema.ts          # 保留
```

## 路由表
```ts
routes: [
  { path: '/', redirect: '/welcome' },
  { path: '/welcome', component: Welcome },       // 空状态
  { path: '/table/:code', name: 'table', component: TableEditor, props: true },
  { path: '/biztype', name: 'biztype', component: BizTypeManage },
  { path: '/enum', name: 'enum', component: EnumManage },
  { path: '/dataset', name: 'dataset', component: DatasetManage },
]
```

## Pinia project store
```ts
export const useProjectStore = defineStore('project', () => {
  const currentProject = ref<Project | null>(null)
  const currentPath = ref('')
  const openedTabs = ref<{ code: string; title: string }[]>([])  // 已打开的表标签
  const activeTab = ref('')

  function openTable(code: string) { ... }  // 加标签 + 路由跳转
  function closeTable(code: string) { ... } // 移标签 + 切换
  function newProject() { ... }
  async function openProject(path) { ... }
  async function saveProject(path?) { ... }
  return { ... }
})
```

## 多标签 + 路由联动
- TabWorkspace: el-tabs v-model=activeTab,tab-remove 关闭
- 切标签 -> router.push(/table/:code)
- <router-view v-slot> + <keep-alive> 缓存表编辑页
- 配置类路由(biztype/enum/dataset)也作为标签打开(单例,不重复开)

## 菜单栏(§6.1)- 原生窗口菜单

**用 Tauri 2 原生菜单**(tauri::menu 内置,无需插件),非网页内菜单。macOS 顶部全局栏 / Windows 窗口菜单。

### Rust 侧(src-tauri/src/lib.rs setup)
```rust
use tauri::menu::{MenuBuilder, SubmenuBuilder, MenuItemBuilder};
use tauri::Emitter;

.setup(|app| {
    let file = SubmenuBuilder::new(app, "文件")
        .text("file.new", "新建项目")
        .text("file.open", "打开项目")
        .text("file.save", "保存")
        .text("file.saveAs", "另存为")
        .separator()
        .quit()
        .build()?;
    let config = SubmenuBuilder::new(app, "配置")
        .text("config.biztype", "业务类型管理")
        .text("config.dataset", "数据集管理")
        .text("config.datasource", "数据源配置")
        .build()?;
    let export = SubmenuBuilder::new(app, "导出")
        .text("export.ddl", "DDL")
        .text("export.diff", "diff")
        .text("export.strconst", "StrConst")
        .build()?;
    let help = SubmenuBuilder::new(app, "帮助")
        .text("help.about", "关于")
        .build()?;
    let menu = MenuBuilder::new(app).items(&[&file,&config,&export,&help]).build()?;
    app.set_menu(menu)?;
    Ok(())
})
.on_menu_event(|app, event| {
    let _ = app.emit("menu", event.id().0.clone());
})
```
菜单项 id 约定: `file.new`/`file.open`/`config.biztype`/`export.ddl` 等。

### 前端侧(监听事件)
```ts
// composables/useMenuActions.ts
import { listen } from '@tauri-apps/api/event'
listen<string>('menu', (e) => {
  switch (e.payload) {
    case 'file.new': projectStore.newProject(); break
    case 'file.open': openProjectDialog(); break
    case 'config.biztype': router.push('/biztype'); break   // 路由标签
    case 'config.datasource': openDataSourceDialog(); break // 弹窗
    case 'export.ddl': openExportDialog('ddl'); break        // 弹窗
    // ...
  }
})
```
`useMenuActions()` 集中把菜单事件映射到 路由跳转 / 弹窗 / store action。
**布局无 MenuBar.vue 组件**(菜单原生)。fe-arch 阶段:菜单项发事件 + 前端 console/占位响应,具体动作各 child 填。

## keep-alive 策略
表编辑页按 code 缓存(include),关闭标签时移除缓存。配置页单例缓存。
