// 监听原生菜单事件,分发到路由跳转 / 弹窗 / store action。
//
// 菜单在 Rust 侧(src-tauri/src/lib.rs)构建,点击后 emit "menu" 事件,
// payload 为菜单项 id(如 "file.new" / "config.biztype")。

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { ElMessage, ElMessageBox } from "element-plus";
import { useRouter } from "vue-router";
import { useProjectStore } from "@/stores/project";
import { useUiStore } from "@/stores/ui";
import { pickOpenFile, pickSaveFile } from "@/composables/useFileDialog";

export function useMenuActions() {
  const router = useRouter();
  const store = useProjectStore();
  const ui = useUiStore();
  let unlisten: UnlistenFn | null = null;

  /** 打开配置类路由标签(单例)。 */
  function openConfigTab(key: string, title: string, path: string) {
    store.openTab({ key, title, path });
    router.push(path);
  }

  async function handle(id: string) {
    switch (id) {
      // 文件
      case "file.new":
        store.newProject();
        router.push("/");
        ElMessage.success("已新建项目");
        break;
      case "file.open":
        await doOpen();
        break;
      case "file.recent":
        ui.openRecent();
        break;
      case "file.save":
        await doSave();
        break;
      case "file.saveAs":
        await doSave(true);
        break;
      case "file.import":
        if (!store.currentProject) {
          ElMessage.warning("请先新建或打开项目");
          break;
        }
        ui.openImport();
        break;
      // 配置(路由标签)
      case "config.biztype":
        openConfigTab("biztype", "业务类型", "/biztype");
        break;
      case "config.enum":
        openConfigTab("enum", "枚举", "/enum");
        break;
      case "config.dataset":
        openConfigTab("dataset", "数据集", "/dataset");
        break;
      // 配置(弹窗)
      case "config.datasource":
        ui.openDataSource();
        break;
      // 导出(弹窗)
      case "export.ddl":
        ui.openExport("ddl");
        break;
      case "export.diff":
        ui.openExport("diff");
        break;
      case "export.strconst":
        ui.openExport("strconst");
        break;
      case "help.guide":
        ElMessageBox.alert(
          [
            "1. 新建/打开项目,或从数据库导入结构",
            "2. 左侧分组树维护表分组,双击表进入编辑",
            "3. 字段页拖拽排序、详情弹窗编辑业务类型/枚举/自动生成",
            "4. 配置菜单维护业务类型、枚举、数据集、数据源",
            "5. 导出菜单生成 DDL / diff / StrConst,Java 页生成实体类",
          ].join("<br/>"),
          "用户指南",
          { confirmButtonText: "确定", dangerouslyUseHTMLString: true }
        );
        break;
      case "help.about":
        ElMessageBox.alert("aqua v2 — JSON-SSOT 数据库结构管理工具", "关于", {
          confirmButtonText: "确定",
        });
        break;
      default:
        break;
    }
  }

  async function doOpen() {
    const path = await pickOpenFile();
    if (!path) return;
    try {
      await store.openProject(path);
      router.push("/");
      ElMessage.success(`已打开 ${path}`);
    } catch {
      /* useTauri 已提示 */
    }
  }

  async function doSave(saveAs = false) {
    if (!store.currentProject) {
      ElMessage.warning("请先新建或打开项目");
      return;
    }
    let path = saveAs ? "" : store.currentPath;
    if (!path) {
      const picked = await pickSaveFile();
      if (!picked) return;
      path = picked;
    }
    try {
      await store.saveProject(path);
      ElMessage.success(`已保存到 ${path}`);
    } catch (e) {
      ElMessage.error(`保存失败: ${e}`);
    }
  }

  /** 挂载监听。在 setup 里调用,返回卸载函数。 */
  async function mount() {
    unlisten = await listen<string>("menu", (e) => handle(e.payload));
  }

  function unmount() {
    unlisten?.();
    unlisten = null;
  }

  return { mount, unmount, handle };
}
