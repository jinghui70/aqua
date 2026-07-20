<script setup lang="ts">
// 欢迎页:操作卡片 + 最近项目列表。
import { ref } from "vue";
import { useRouter } from "vue-router";
import { ElMessage } from "element-plus";
import { useProjectStore } from "@/stores/project";
import { useUiStore } from "@/stores/ui";
import { useRecentProjects, type RecentProject } from "@/composables/useRecentProjects";
import { pickOpenFile } from "@/composables/useFileDialog";

const router = useRouter();
const store = useProjectStore();
const ui = useUiStore();
const recent = useRecentProjects();

const recentList = ref<RecentProject[]>(recent.load());

function refresh() {
  recentList.value = recent.load();
}

async function handleNew() {
  if (!(await store.confirmIfDirty())) return;
  ui.openNewProject();
}

async function handleOpen() {
  const path = await pickOpenFile();
  if (path) await openPath(path);
}

async function openPath(path: string) {
  if (!(await store.confirmIfDirty())) return;
  try {
    await store.openProject(path);
    ElMessage.success(`已打开 ${path}`);
    refresh();
  } catch {
    // 打开失败(文件可能已删),从最近列表移除
    recent.remove(path);
    refresh();
  }
}

function removeRecent(path: string) {
  recent.remove(path);
  refresh();
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
  <div class="h-full flex flex-col items-center justify-center px-40">
    <div class="w-full max-w-720">
      <!-- 标题 -->
      <div class="text-center mb-40">
        <div class="text-32 font-bold text-gray-700">Aqua</div>
        <div class="text-14 text-gray-400 mt-8">JSON-SSOT 数据库结构设计</div>
      </div>

      <!-- 操作卡片 -->
      <div class="flex gap-16 mb-40">
        <div
          class="flex-1 flex flex-col items-center py-24 border border-gray-200 rounded-8 cursor-pointer hover:border-blue-400 hover:shadow transition"
          @click="handleNew"
        >
          <div class="text-32 mb-8">📄</div>
          <div class="text-14 font-bold">新建项目</div>
        </div>
        <div
          class="flex-1 flex flex-col items-center py-24 border border-gray-200 rounded-8 cursor-pointer hover:border-blue-400 hover:shadow transition"
          @click="handleOpen"
        >
          <div class="text-32 mb-8">📂</div>
          <div class="text-14 font-bold">打开项目</div>
        </div>
      </div>

      <!-- 最近项目 -->
      <div>
        <div class="text-13 font-bold text-gray-500 mb-12">最近项目</div>
        <div v-if="recentList.length" class="flex flex-col gap-4">
          <div
            v-for="r in recentList"
            :key="r.path"
            class="flex items-center justify-between px-12 py-8 rounded-6 cursor-pointer hover:bg-gray-100 group"
            @click="openPath(r.path)"
          >
            <div class="flex flex-col">
              <span class="text-13">{{ r.name ?? basename(r.path) }}</span>
              <span class="text-12 text-gray-400">{{ r.path }}</span>
            </div>
            <div class="flex items-center gap-12">
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
        <div v-else class="text-13 text-gray-400 py-16 text-center">
          暂无最近项目
        </div>
      </div>
    </div>
  </div>
</template>
