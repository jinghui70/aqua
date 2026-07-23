<script setup lang="ts">
// 字符串变量导出:即时预览 + 复制/保存。包名 = basePackage(全部)/ basePackage.group(按分组)。
import { computed, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useUiStore } from "@/stores/ui";
import { useProjectStore } from "@/stores/project";
import { useTauri } from "@/composables/useTauri";

const ui = useUiStore();
const store = useProjectStore();
const tauri = useTauri();

const scope = ref<"all" | "group">("group");
const selectedGroup = ref("");
const preview = ref("");

const groups = computed(() => store.currentProject?.groups ?? []);

async function doPreview() {
  if (!store.currentProject) return;
  try {
    preview.value = await tauri.generateStrConst(store.currentProject, {
      group: scope.value === "group" ? selectedGroup.value : undefined,
    });
  } catch { /* 已提示 */ }
}

async function copy() {
  await navigator.clipboard.writeText(preview.value);
  ElMessage.success("已复制");
}

async function saveFile() {
  const path = await save({
    filters: [{ name: "Java", extensions: ["java"] }],
    defaultPath: "DatabaseConstants.java",
  });
  if (!path) return;
  try {
    await invoke<void>("write_text_file", { path, content: preview.value });
    ElMessage.success("已保存");
  } catch (e) {
    ElMessage.error(`保存失败: ${e}`);
  }
}

// 每次打开重置:默认按分组(选第一个)
watch(() => ui.strConstExportVisible, (v) => {
  if (v) {
    preview.value = "";
    scope.value = "group";
    selectedGroup.value = groups.value[0]?.code ?? "";
  }
});
// 即时预览
watch([scope, selectedGroup, () => ui.strConstExportVisible], () => {
  if (ui.strConstExportVisible) doPreview();
});
</script>

<template>
  <el-dialog v-model="ui.strConstExportVisible" title="导出字符串变量" width="70%" :close-on-click-modal="false">
    <div class="flex flex-col gap-12">
      <div class="flex items-center">
        <el-radio-group v-model="scope" size="small">
          <el-radio-button value="all">全部表</el-radio-button>
          <el-radio-button value="group">按分组</el-radio-button>
        </el-radio-group>
        <el-select
          v-if="scope === 'group'"
          v-model="selectedGroup"
          size="small"
          placeholder="选分组"
          style="width: 140px; margin-left: 8px"
        >
          <el-option v-for="g in groups" :key="g.code" :label="g.name" :value="g.code" />
        </el-select>
        <div class="flex-1" />
        <el-button size="small" @click="copy" :disabled="!preview" style="margin-right: 8px">复制</el-button>
        <el-button size="small" type="primary" @click="saveFile" :disabled="!preview">保存</el-button>
      </div>
      <el-input v-model="preview" type="textarea" :rows="20" readonly class="font-mono" />
    </div>
  </el-dialog>
</template>
