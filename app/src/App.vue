<script setup lang="ts">
import { onMounted, onUnmounted, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { exit } from "@tauri-apps/plugin-process";
import AppLayout from "@/layout/AppLayout.vue";
import { useMenuActions } from "@/composables/useMenuActions";
import { useBuiltinStore } from "@/stores/builtin";
import { useDatabaseStore } from "@/stores/database";
import { useProjectStore } from "@/stores/project";

const menu = useMenuActions();
const builtin = useBuiltinStore();
const database = useDatabaseStore();
const store = useProjectStore();
let unlistenClose: (() => void) | null = null;
let unlistenExit: (() => void) | null = null;

// 窗口标题随项目变:有项目 -> "Aqua-中文名"(名字空退回 Aqua),无项目 -> "Aqua"。
// 覆盖打开/新建/改名/关闭所有场景(watch name,immediate 含初始态)。
watch(
  () => store.currentProject?.name,
  (name) => {
    const title = store.currentProject ? (name ? `Aqua-${name}` : "Aqua") : "Aqua";
    void getCurrentWindow().setTitle(title);
  },
  { immediate: true }
);

// 退出确认:dirty 时弹保存/不保存/取消;保存/不保存 -> 标记已确认 + exit;取消 -> 不退
async function doConfirmExit() {
  if (!store.dirty) {
    await invoke("set_exit_confirmed");
    await exit(0);
    return;
  }
  const ok = await store.confirmIfDirty();
  if (ok) {
    await invoke("set_exit_confirmed");
    await exit(0);
  }
}

onMounted(async () => {
  void builtin.load();
  void database.load();
  menu.mount();
  // 窗口 X 关闭:拦截 + 确认
  unlistenClose = await getCurrentWindow().onCloseRequested(async (event) => {
    event.preventDefault();
    await doConfirmExit();
  });
  // Command+Q/菜单 quit:Rust ExitRequested 拦截后 emit,前端确认
  unlistenExit = await listen("confirm-exit", doConfirmExit);
});
onUnmounted(() => {
  menu.unmount();
  unlistenClose?.();
  unlistenExit?.();
});
</script>

<template>
  <AppLayout />
</template>
