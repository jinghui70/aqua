<script setup lang="ts">
// 选表对话框:树形(分组->表)+ 多选 + 过滤。confirm 返回选中表 code 列表。
import { computed, nextTick, ref, watch } from "vue";
import { useProjectStore } from "@/stores/project";

const props = defineProps<{ modelValue: boolean; selected: string[] }>();
const emit = defineEmits<{ "update:modelValue": [boolean]; confirm: [string[]] }>();

const store = useProjectStore();
const treeRef = ref();
const filterText = ref("");

const treeData = computed(() => [
  {
    code: "root",
    name: "全部表",
    children: (store.currentProject?.groups ?? []).map((g) => ({
      code: `g:${g.code}`,
      name: g.name,
      children: (store.currentProject?.tables ?? [])
        .filter((t) => t.group === g.code)
        .map((t) => ({ code: t.code, name: `${t.code} (${t.name})` })),
    })),
  },
]);

watch(
  () => props.modelValue,
  (v) => {
    if (v) {
      filterText.value = "";
      nextTick(() => treeRef.value?.setCheckedKeys(props.selected));
    }
  }
);

watch(filterText, (val) => treeRef.value?.filter(val));

function filterNode(value: string, data: any) {
  if (!value) return true;
  return (data.name as string).toLowerCase().includes(value.toLowerCase());
}

function confirm() {
  const checked = treeRef.value?.getCheckedKeys() ?? [];
  // 只取表 code(非 g: 前缀的分组节点)
  const tables = checked.filter((k: string) => k !== "root" && !k.startsWith("g:"));
  emit("confirm", tables);
  emit("update:modelValue", false);
}

function cancel() {
  emit("update:modelValue", false);
}
</script>

<template>
  <el-dialog :model-value="modelValue" title="选择表" width="480px" :close-on-click-modal="false" @update:model-value="emit('update:modelValue', $event)">
    <el-input v-model="filterText" placeholder="过滤表名" clearable size="small" class="mb-8" />
    <div style="max-height: 360px; overflow-y: auto">
      <el-tree
        ref="treeRef"
        :data="treeData"
        :props="{ label: 'name', children: 'children' }"
        node-key="code"
        show-checkbox
        :filter-node-method="filterNode"
        default-expand-all
      />
    </div>
    <template #footer>
      <el-button @click="cancel">取消</el-button>
      <el-button type="primary" @click="confirm">确定</el-button>
    </template>
  </el-dialog>
</template>
