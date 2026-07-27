<script setup lang="ts">
// 多标签工作区:el-tabs 驱动路由 + keep-alive 缓存已打开页。
//
// 关闭 tab 时该 path 的 closedCount 递增,:key 拼上 rev -> keep-alive 视作不同 key,
// 重开即新实例(否则命中旧缓存显示上次状态)。:max 兜底防旧缓存无限堆积。
import { computed } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useProjectStore } from "@/stores/project";

const router = useRouter();
const route = useRoute();
const store = useProjectStore();

const activeKey = computed({
  get: () => store.activeTab,
  set: (key: string) => {
    const tab = store.openedTabs.find((t) => t.key === key);
    if (tab) {
      store.activeTab = key;
      router.push(tab.path);
    }
  },
});

function onTabRemove(key: string) {
  const nextPath = store.closeTab(key);
  if (nextPath) router.push(nextPath);
}

// :key = fullPath + 关闭计数;关闭后再开 rev 变 -> keep-alive 不命中旧缓存 -> 新实例
const cacheKey = computed(() => {
  const rev = store.closedCount[route.fullPath] ?? 0;
  return rev ? `${route.fullPath}#${rev}` : route.fullPath;
});
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden">
    <el-tabs
      v-if="store.openedTabs.length"
      v-model="activeKey"
      type="card"
      closable
      class="workspace-tabs flex-shrink-0"
      @tab-remove="onTabRemove"
    >
      <el-tab-pane
        v-for="tab in store.openedTabs"
        :key="tab.key"
        :name="tab.key"
        :label="tab.title"
      />
    </el-tabs>
    <div class="flex-1 min-h-0 overflow-hidden">
      <router-view v-slot="{ Component }">
        <keep-alive :max="30">
          <component :is="Component" :key="cacheKey" />
        </keep-alive>
      </router-view>
    </div>
  </div>
</template>

<style scoped>
.workspace-tabs :deep(.el-tabs__header) {
  margin: 0;
}
</style>
