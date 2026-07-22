<script setup lang="ts">
// 配置中心:项目级配置(项目设置/数据源/业务类型)统一管理。左侧返回 + 导航 + 右侧面板。
import { ref } from "vue";
import { useRouter } from "vue-router";
import ProjectSettingsPanel from "./config/ProjectSettingsPanel.vue";
import DataSourcePanel from "./config/DataSourcePanel.vue";
import BizTypePanel from "./config/BizTypePanel.vue";
import AutoGenStrategyPanel from "./config/AutoGenStrategyPanel.vue";

const router = useRouter();
const activePanel = ref<"settings" | "datasource" | "biztype" | "strategy">("settings");
</script>

<template>
  <div class="h-full flex flex-col">
    <div class="flex items-center gap-8 px-12 py-6 border-b border-gray-200 flex-shrink-0">
      <el-button size="small" link @click="router.push('/')">← 返回</el-button>
      <span class="text-14 font-bold">配置</span>
    </div>
    <div class="flex-1 min-h-0 flex">
      <div class="w-160 border-r border-gray-200 flex-shrink-0 overflow-auto">
        <el-menu :default-active="activePanel" @select="(k: string) => (activePanel = k as typeof activePanel)">
          <el-menu-item index="settings">项目设置</el-menu-item>
          <el-menu-item index="datasource">数据源</el-menu-item>
          <el-menu-item index="biztype">业务类型</el-menu-item>
          <el-menu-item index="strategy">自动生成策略</el-menu-item>
        </el-menu>
      </div>
      <div class="flex-1 min-h-0 overflow-auto p-12">
        <ProjectSettingsPanel v-if="activePanel === 'settings'" />
        <DataSourcePanel v-else-if="activePanel === 'datasource'" />
        <BizTypePanel v-else-if="activePanel === 'biztype'" />
        <AutoGenStrategyPanel v-else-if="activePanel === 'strategy'" />
      </div>
    </div>
  </div>
</template>
