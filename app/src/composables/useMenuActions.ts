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

  async function handle(id: string) {
    // 对话框打开时忽略菜单事件(模态:原生菜单栏无法灰显,但操作不执行,避免打断)
    if (ui.anyDialogOpen) return;
    switch (id) {
      // 文件
      case "file.new":
        if (!(await store.confirmIfDirty())) break;
        ui.openNewProject();
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
      case "file.close":
        if (await store.closeProject()) router.push("/welcome");
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
    if (!(await store.confirmIfDirty())) return;
    try {
      await store.openProject(path);
      router.push("/");
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

  return { mount, unmount, handle, doSave };
}
