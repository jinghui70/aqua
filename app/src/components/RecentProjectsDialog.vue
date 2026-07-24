<script setup lang="ts">
// 最近项目对话框:菜单「文件 → 最近项目」触发,列出并打开最近项目。
import { ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useProjectStore } from "@/stores/project";
import { useUiStore } from "@/stores/ui";
import { useRecentProjects, type RecentProject } from "@/composables/useRecentProjects";

const router = useRouter();
const store = useProjectStore();
const ui = useUiStore();
const recent = useRecentProjects();

const list = ref<RecentProject[]>([]);

// 打开对话框时刷新列表
watch(
  () => ui.recentVisible,
  (v) => {
    if (v) list.value = recent.load();
  }
);

async function openPath(path: string) {
  if (!(await store.confirmIfDirty())) return;
  try {
    await store.openProject(path);
    router.push("/");
    ui.recentVisible = false;
  } catch {
    // 打开失败(文件可能已删),从最近列表移除
    recent.remove(path);
    list.value = recent.load();
  }
}

function removeRecent(path: string) {
  recent.remove(path);
  list.value = recent.load();
}

function fmtTime(ts: number): string {
  const d = new Date(ts);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function basename(path: string): string {
  return path.split("/").pop() || path;
}
</script>

<template>
  <el-dialog v-model="ui.recentVisible" title="最近项目" width="560px" :close-on-click-modal="false">
    <div v-if="list.length" class="flex flex-col gap-4">
      <div
        v-for="r in list"
        :key="r.path"
        class="flex items-center justify-between px-12 py-8 rounded-6 cursor-pointer hover:bg-gray-100 group"
        @click="openPath(r.path)"
      >
        <div class="flex flex-col min-w-0">
          <span class="text-13">{{ r.name ?? basename(r.path) }}</span>
          <span class="text-12 text-gray-400 truncate">{{ r.path }}</span>
        </div>
        <div class="flex items-center gap-12 flex-shrink-0">
          <span class="text-12 text-gray-400">{{ fmtTime(r.openedAt) }}</span>
          <el-button
            size="small"
            link
            type="danger"
            class="opacity-0 group-hover:opacity-100"
            @click.stop="removeRecent(r.path)"
          >
            移除
          </el-button>
        </div>
      </div>
    </div>
    <div v-else class="text-13 text-gray-400 py-16 text-center">暂无最近项目</div>
  </el-dialog>
</template>
