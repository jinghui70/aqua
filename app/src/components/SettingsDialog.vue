<script setup lang="ts">
// 设置对话框(Mac 风格 Cmd+,):左侧分类导航 + 右侧面板。替代原全屏 /config 路由页。
// 五个分类复用 views/config/ 下的 Panel 组件;驱动管理并入(原独立弹窗)。
import { ref, watch } from "vue";
import { useUiStore, type SettingsPanel } from "@/stores/ui";
import ProjectSettingsPanel from "@/views/config/ProjectSettingsPanel.vue";
import DataSourcePanel from "@/views/config/DataSourcePanel.vue";
import BizTypePanel from "@/views/config/BizTypePanel.vue";
import AutoGenStrategyPanel from "@/views/config/AutoGenStrategyPanel.vue";
import DriverPanel from "@/views/config/DriverPanel.vue";

const ui = useUiStore();

const activePanel = ref<SettingsPanel>("settings");

// 打开时同步到 store 指定的分类(支持从入口直接定位,如驱动管理)
watch(
  () => ui.settingsVisible,
  (visible) => {
    if (visible) activePanel.value = ui.settingsPanel;
  }
);

const NAV: { key: SettingsPanel; label: string }[] = [
  { key: "settings", label: "项目设置" },
  { key: "datasource", label: "数据源" },
  { key: "biztype", label: "业务类型" },
  { key: "strategy", label: "自动生成策略" },
  { key: "driver", label: "驱动管理" },
];
</script>

<template>
  <el-dialog
    v-model="ui.settingsVisible"
    title="设置"
    width="900px"
    top="6vh"
    :close-on-click-modal="false"
    destroy-on-close
    class="settings-dialog"
  >
    <div class="flex h-560">
      <!-- 左导航 -->
      <div class="w-160 border-r border-gray-200 flex-shrink-0 overflow-auto">
        <el-menu :default-active="activePanel" @select="(k: string) => (activePanel = k as SettingsPanel)">
          <el-menu-item v-for="n in NAV" :key="n.key" :index="n.key">{{ n.label }}</el-menu-item>
        </el-menu>
      </div>
      <!-- 右面板 -->
      <div class="flex-1 min-w-0 overflow-auto p-16">
        <ProjectSettingsPanel v-if="activePanel === 'settings'" />
        <DataSourcePanel v-else-if="activePanel === 'datasource'" />
        <BizTypePanel v-else-if="activePanel === 'biztype'" />
        <AutoGenStrategyPanel v-else-if="activePanel === 'strategy'" />
        <DriverPanel v-else-if="activePanel === 'driver'" />
      </div>
    </div>
  </el-dialog>
</template>

<style scoped>
/* 设置类对话框:body 归零 padding,由内部左右分栏自行控制布局 */
.settings-dialog :deep(.el-dialog__body) {
  padding: 0;
}
</style>
