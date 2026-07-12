<script setup lang="ts">
// 多标签工作区:el-tabs 驱动路由 + keep-alive 缓存已打开页。
import { computed } from "vue";
import { useRouter } from "vue-router";
import { useProjectStore } from "@/stores/project";

const router = useRouter();
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
      <router-view v-slot="{ Component, route }">
        <keep-alive>
          <component :is="Component" :key="route.fullPath" />
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
