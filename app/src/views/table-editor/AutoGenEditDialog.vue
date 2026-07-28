<script setup lang="ts">
// 自动生成编辑弹窗: 开关 + 策略 + 时机 + 策略参数。
// 补齐行内表格放不下的 autoGenerate 属性。draft 副本编辑,保存写回。
import { computed, ref, watch } from "vue";
import type { Field, AutoGenStrategyDefine } from "@/types/schema";
import { useProjectStore } from "@/stores/project";
import { useBuiltinStore } from "@/stores/builtin";

const props = defineProps<{ modelValue: boolean; field: Field | null }>();
const emit = defineEmits<{ "update:modelValue": [boolean] }>();

const store = useProjectStore();
const builtin = useBuiltinStore();

const visible = computed({
  get: () => props.modelValue,
  set: (v) => emit("update:modelValue", v),
});

// 本地编辑副本(确认时写回)
const draft = ref<Field | null>(null);

// 自动生成策略(内置 + 自定义),策略下拉 + param 条件显示
const autoGenStrategies = computed<AutoGenStrategyDefine[]>(() => [
  ...builtin.autoGenStrategies,
  ...(store.currentProject?.autoGenStrategies ?? []),
]);
const currentStrategy = computed(() =>
  autoGenStrategies.value.find((s) => s.code === draft.value?.autoGenerate?.strategy)
);

// 打开时重建 draft(深拷贝,取消不污染原对象)
watch(visible, (v) => {
  if (v && props.field) {
    draft.value = JSON.parse(JSON.stringify(props.field));
  }
});

// ===== 保存 =====
function save() {
  if (!draft.value || !props.field) return;
  // 写回原字段(保持引用,Object.assign);本弹窗不编辑 code,无 code 级联
  Object.keys(props.field).forEach((k) => delete (props.field as any)[k]);
  Object.assign(props.field, draft.value);
  visible.value = false;
}
</script>

<template>
  <el-dialog v-model="visible" title="自动生成" width="560px" :close-on-click-modal="false">
    <el-form v-if="draft" label-width="90px" class="pr-12" :disabled="store.readOnly">
      <el-form-item label="自动生成">
        <el-switch
          :model-value="!!draft.autoGenerate"
          @change="(v: any) => draft && (draft.autoGenerate = v ? { strategy: 'default', timing: 'INSERT' } : undefined)"
        />
      </el-form-item>
      <template v-if="draft.autoGenerate">
        <div class="grid grid-cols-2 gap-x-24">
          <el-form-item label="生成策略">
            <el-select v-model="draft.autoGenerate.strategy">
              <el-option v-for="s in autoGenStrategies" :key="s.code" :label="s.name" :value="s.code" />
            </el-select>
          </el-form-item>
          <el-form-item label="生成时机">
            <el-select v-model="draft.autoGenerate.timing">
              <el-option label="INSERT" value="INSERT" />
              <el-option label="INSERT_UPDATE" value="INSERT_UPDATE" />
            </el-select>
          </el-form-item>
        </div>
        <el-form-item v-if="currentStrategy?.paramDesc != null" label="策略参数">
          <el-input v-model="draft.autoGenerate.param" :placeholder="currentStrategy.paramDesc" />
        </el-form-item>
      </template>
    </el-form>

    <template #footer>
      <el-button @click="visible = false">取消</el-button>
      <el-button type="primary" :disabled="store.readOnly" @click="save">保存</el-button>
    </template>
  </el-dialog>
</template>
