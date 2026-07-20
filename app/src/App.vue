<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
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

onMounted(async () => {
  void builtin.load();
  void database.load();
  menu.mount();
  // 关闭应用前检查 dirty:有未保存改动时提示保存/不保存/取消
  unlistenClose = await getCurrentWindow().onCloseRequested(async (event) => {
    if (!store.dirty) return; // 无未保存改动,正常关闭
    event.preventDefault();
    const ok = await store.confirmIfDirty();
    if (ok) {
      await getCurrentWindow().destroy(); // 保存/不保存,继续关闭
    }
    // 取消则保持打开
  });
});
onUnmounted(() => {
  menu.unmount();
  unlistenClose?.();
});
</script>

<template>
  <AppLayout />
</template>
