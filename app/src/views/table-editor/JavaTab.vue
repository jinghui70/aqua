<script setup lang="ts">
// java Tab: 配置(包名/类名/Lombok/注释)+ 实时预览 + 复制/保存。
import { ref, computed, watch } from "vue";
import { ElMessage } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useTauri } from "@/composables/useTauri";
import { useProjectStore } from "@/stores/project";
import { snakeToPascal } from "@/composables/useNaming";

const props = defineProps<{ tableCode: string; active: boolean }>();

const tauri = useTauri();
const store = useProjectStore();

const useLombok = ref(true);
const packageName = ref("");
const className = ref("");
const preview = ref("");

// 当前表的默认包名 {basePackage}.{group}.entity(对齐后端 default_package)
const defaultPackage = computed(() => {
  const p = store.currentProject;
  if (!p) return "";
  const table = p.tables.find((t) => t.code === props.tableCode);
  const group = (table?.group ?? "").toLowerCase();
  return `${p.basePackage}.${group}.entity`;
});
// 类名占位符:表 code 派生的大驼峰(为空时显示,提示默认值)
const classNamePlaceholder = computed(() => snakeToPascal(props.tableCode));

async function refresh() {
  if (!store.currentProject) return;
  try {
    preview.value = await tauri.generateJava(
      store.currentProject,
      props.tableCode,
      {
        useLombok: useLombok.value,
        package: packageName.value || undefined,
        className: className.value || undefined,
      }
    );
  } catch {
    /* 已提示 */
  }
}

// 切表:包名重置为该表默认值(预填,可改),类名清空(用 placeholder 提示)
watch(
  () => props.tableCode,
  () => {
    packageName.value = defaultPackage.value;
    className.value = "";
  },
  { immediate: true }
);
// 配置变化实时刷新
watch([useLombok, packageName, className], refresh, { immediate: true });
// 切回本 tab 时重新生成,同步字段/索引的改动
watch(() => props.active, (a) => a && refresh());

async function copy() {
  await navigator.clipboard.writeText(preview.value);
  ElMessage.success("已复制");
}

async function saveFile() {
  // 类名默认派生 PascalCase(与后端生成的 class 名一致),而非裸表名
  const cls = className.value || snakeToPascal(props.tableCode);
  const path = await save({
    filters: [{ name: "Java", extensions: ["java"] }],
    defaultPath: `${cls}.java`,
  });
  if (!path) return;
  try {
    await invoke<void>("write_text_file", { path, content: preview.value });
    ElMessage.success("已保存");
  } catch (e) {
    ElMessage.error(`保存失败: ${e}`);
  }
}
</script>

<template>
  <div class="h-full flex flex-col gap-12">
    <div class="flex items-center gap-16 flex-wrap flex-shrink-0">
      <span class="text-13">
        包名
        <el-input
          v-model="packageName"
          size="small"
          placeholder="默认 basePackage.group.entity"
          style="width: 260px"
        />
      </span>
      <span class="text-13">
        类名
        <el-input
          v-model="className"
          size="small"
          :placeholder="classNamePlaceholder"
          style="width: 140px"
        />
      </span>
      <el-checkbox v-model="useLombok">Lombok</el-checkbox>
      <div class="flex-1" />
      <el-button size="small" @click="copy">复制</el-button>
      <el-button size="small" type="primary" @click="saveFile">保存</el-button>
    </div>
    <div class="flex-1 min-h-0">
      <el-input
        v-model="preview"
        type="textarea"
        resize="none"
        readonly
        class="font-mono h-full"
        :input-style="{ height: '100%' }"
      />
    </div>
  </div>
</template>
