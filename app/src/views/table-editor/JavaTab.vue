<script setup lang="ts">
// java Tab: 配置(包名/类名/Lombok/注释)+ 实时预览 + 复制/保存。
// 多文件:实体 + 本表定义枚举,切换文件预览,逐个保存。
import { ref, computed, watch } from "vue";
import { ElMessage } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useTauri } from "@/composables/useTauri";
import { useProjectStore } from "@/stores/project";
import { snakeToPascal } from "@/composables/useNaming";
import type { JavaFile } from "@/types/schema";

const props = defineProps<{ tableCode: string; active: boolean }>();

const tauri = useTauri();
const store = useProjectStore();

const useLombok = ref(true);
const pkg = ref(""); // 完整包名,空则不生成 package 声明
const className = ref("");
const files = ref<JavaFile[]>([]);
const activeFile = ref(""); // 当前预览文件名(path)
const preview = computed(
  () => files.value.find((f) => f.path === activeFile.value)?.content ?? ""
);

// 默认包名 = basePackage.{group}.entity(basePackage 为空则 {group}.entity)
const defaultPackage = computed(() => {
  const base = store.currentProject?.basePackage ?? "";
  const table = store.currentProject?.tables.find((t) => t.code === props.tableCode);
  const group = (table?.group ?? "").toLowerCase();
  const suffix = group ? `${group}.entity` : "entity";
  return base ? `${base}.${suffix}` : suffix;
});
// 当前表(schema 内)
const currentTable = computed(() =>
  store.currentProject?.tables.find((t) => t.code === props.tableCode)
);
// 类名占位符:表 code 派生的大驼峰(为空时显示,提示默认值)
const classNamePlaceholder = computed(() => snakeToPascal(props.tableCode));

async function refresh() {
  if (!store.currentProject) return;
  // 持久化包名:把当前输入写回表 javaPackage(与生成同步,引用枚举 import 按它取)
  if (currentTable.value) {
    currentTable.value.javaPackage = pkg.value || undefined;
  }
  try {
    const list = await tauri.generateJava(
      store.currentProject,
      props.tableCode,
      {
        useLombok: useLombok.value,
        package: pkg.value || undefined,
        className: className.value || undefined,
      }
    );
    files.value = list;
    // 保留当前选中文件,否则默认首个(实体)
    if (!list.some((f) => f.path === activeFile.value)) {
      activeFile.value = list[0]?.path ?? "";
    }
  } catch {
    files.value = [];
  }
}

// 切表:包名优先用该表持久化的 javaPackage(改过则记住),否则默认值;类名清空
watch(
  () => props.tableCode,
  () => {
    pkg.value = currentTable.value?.javaPackage || defaultPackage.value;
    className.value = "";
    activeFile.value = "";
  },
  { immediate: true }
);
// 配置变化实时刷新
watch([useLombok, pkg, className], refresh, { immediate: true });
// 切回本 tab 时重新生成,同步字段/索引的改动
watch(() => props.active, (a) => a && refresh());

function resetClassName() {
  className.value = "";
}

async function copy() {
  await navigator.clipboard.writeText(preview.value);
  ElMessage.success("已复制");
}

async function saveFile() {
  const file = files.value.find((f) => f.path === activeFile.value);
  if (!file) return;
  const path = await save({
    filters: [{ name: "Java", extensions: ["java"] }],
    defaultPath: file.path,
  });
  if (!path) return;
  try {
    await invoke<void>("write_text_file", { path, content: file.content });
    ElMessage.success("已保存");
  } catch (e) {
    ElMessage.error(`保存失败: ${e}`);
  }
}
</script>

<template>
  <div class="h-full flex flex-col gap-12">
    <div class="flex items-center gap-12 flex-wrap flex-shrink-0">
      <span class="text-13">
        包名
        <el-input
          v-model="pkg"
          size="small"
          style="width: 300px"
        />
      </span>
      <span class="text-13">
        类名
        <el-input
          v-model="className"
          size="small"
          clearable
          :placeholder="classNamePlaceholder"
          style="width: 140px"
          @clear="resetClassName"
        />
      </span>
      <el-checkbox v-model="useLombok">Lombok</el-checkbox>
      <div class="flex-1" />
      <el-button size="small" @click="copy">复制</el-button>
      <el-button size="small" type="primary" @click="saveFile">保存</el-button>
    </div>
    <!-- 多文件切换(实体 + 枚举) -->
    <div v-if="files.length > 1" class="flex gap-8 flex-shrink-0">
      <el-check-tag
        v-for="f in files"
        :key="f.path"
        :checked="f.path === activeFile"
        @change="activeFile = f.path"
        class="cursor-pointer"
      >
        {{ f.path }}
      </el-check-tag>
      <span class="text-12 text-gray-400 self-center">{{ files.length }} 个文件</span>
    </div>
    <div class="flex-1 min-h-0">
      <el-input
        :model-value="preview"
        type="textarea"
        resize="none"
        readonly
        class="font-mono h-full"
        :input-style="{ height: '100%' }"
      />
    </div>
  </div>
</template>
