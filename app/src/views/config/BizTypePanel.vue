<script setup lang="ts">
// 业务类型管理(§6.5):左列表 + 右编辑。常驻配置中心面板。
import { computed, nextTick, ref, watch } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import Sortable from "sortablejs";
import { useProjectStore } from "@/stores/project";
import { useBuiltinStore } from "@/stores/builtin";
import { collectRelatedTables, buildCascadePrompt } from "@/utils/cascade";
import { DataType, type BizTypeDefine } from "@/types/schema";

const store = useProjectStore();
const builtin = useBuiltinStore();
const dataTypes = Object.values(DataType);

const projectBizTypes = computed(() => store.currentProject?.bizTypes ?? []);
// 合并展示:内置(只读)+ 自定义(可改)
const bizTypes = computed<BizTypeDefine[]>(() => [
  ...builtin.bizTypes,
  ...projectBizTypes.value,
]);
const selectedCode = ref("");
const current = computed(() =>
  bizTypes.value.find((b) => b.bizType === selectedCode.value)
);
const isCurrentBuiltin = computed(() =>
  selectedCode.value ? builtin.isBuiltin(selectedCode.value) : false
);
// 两个只读来源:预置内置 + 全局只读。任一 -> 只读(name/描述用 readonly,表格用 span)
const isReadonly = computed(() => isCurrentBuiltin.value || store.readOnly);

function select(code: string) {
  selectedCode.value = code;
}

// ===== 新建:ElDialog 录 code + name =====
const newBizVisible = ref(false);
const newCode = ref("");
const newName = ref("");
function addBizType() {
  if (!store.currentProject) {
    ElMessage.warning("请先打开项目");
    return;
  }
  newCode.value = "";
  newName.value = "";
  newBizVisible.value = true;
}
function confirmNewBiz() {
  const code = newCode.value.trim();
  if (!code) {
    ElMessage.warning("code 不能为空");
    return;
  }
  if (bizTypes.value.some((b) => b.bizType === code)) {
    ElMessage.error(`${code} 已存在`);
    return;
  }
  const biz: BizTypeDefine = {
    bizType: code,
    name: newName.value.trim() || code,
    supportedDataTypes: [],
  };
  store.currentProject!.bizTypes.push(biz);
  selectedCode.value = code;
  newBizVisible.value = false;
  ElMessage.success("已创建");
}

async function removeBizType(code: string) {
  if (builtin.isBuiltin(code)) return;
  const related = collectRelatedTables(store.currentProject, (f) => f.bizType === code);
  const msg = buildCascadePrompt("业务类型", code, related);
  try {
    await ElMessageBox.confirm(msg, "删除业务类型", {
      type: "warning",
      confirmButtonText: "删除",
      cancelButtonText: "取消",
      dangerouslyUseHTMLString: true,
    });
    for (const t of store.currentProject!.tables) {
      for (const f of t.fields) {
        if (f.bizType === code) {
          f.bizType = undefined;
          f.bizTypeData = undefined;
        }
      }
    }
    const arr = store.currentProject!.bizTypes;
    const idx = arr.findIndex((b) => b.bizType === code);
    if (idx >= 0) arr.splice(idx, 1);
    if (selectedCode.value === code) selectedCode.value = "";
    ElMessage.success(
      related.length ? `已删除,并清除 ${related.length} 张表的关联` : "已删除"
    );
  } catch {
    /* 取消 */
  }
}

// supportedDataTypes 子表
function addSupported() {
  current.value?.supportedDataTypes.push({ dataType: DataType.Varchar });
}
function removeSupported(idx: number) {
  current.value?.supportedDataTypes.splice(idx, 1);
}

// bizTypeData.fields 子表
function ensureBizTypeData() {
  if (current.value && !current.value.bizTypeData) {
    current.value.bizTypeData = { fields: [] };
  }
}
function addDataField() {
  ensureBizTypeData();
  current.value!.bizTypeData!.fields.push({ name: "", type: "string" });
}
function removeDataField(idx: number) {
  current.value?.bizTypeData?.fields.splice(idx, 1);
}

// ===== 拖拽排序(数据类型 + 参数)=====
const supportedTableRef = ref();
const dataFieldTableRef = ref();
let supportedSortable: Sortable | null = null;
let dataFieldSortable: Sortable | null = null;
function bindSortable(refEl: any, arr: () => any[] | undefined): Sortable | null {
  const tbody = refEl?.$el?.querySelector(".el-table__body-wrapper tbody");
  if (!tbody) return null;
  return Sortable.create(tbody, {
    handle: ".drag-handle",
    animation: 150,
    forceFallback: true,
    fallbackOnBody: true,
    disabled: isReadonly.value,
    onEnd({ oldIndex, newIndex }) {
      if (oldIndex == null || newIndex == null || oldIndex === newIndex) return;
      nextTick(() => {
        const a = arr();
        if (!a) return;
        const [moved] = a.splice(oldIndex, 1);
        a.splice(newIndex, 0, moved);
      });
    },
  });
}
// current 有值后(表格渲染)绑定 Sortable
watch(current, (c) => {
  if (!c) return;
  nextTick(() => {
    if (!supportedSortable) supportedSortable = bindSortable(supportedTableRef, () => current.value?.supportedDataTypes);
    if (!dataFieldSortable) dataFieldSortable = bindSortable(dataFieldTableRef, () => current.value?.bizTypeData?.fields);
  });
}, { immediate: true });
watch(isReadonly, (ro) => {
  supportedSortable?.option("disabled", ro);
  dataFieldSortable?.option("disabled", ro);
});
</script>

<template>
  <div v-if="store.currentProject" class="h-full flex">
    <!-- 左列表 -->
    <div class="w-220 border-r border-gray-200 flex flex-col flex-shrink-0">
      <div
        class="flex items-center justify-between px-12 h-40 border-b border-gray-200 font-bold text-14"
      >
        <span>业务类型</span>
        <el-button
          v-if="!store.readOnly"
          size="small"
          type="primary"
          link
          @click="addBizType"
        >+ 新建</el-button>
      </div>
      <div class="flex-1 overflow-y-auto">
        <div
          v-for="b in bizTypes"
          :key="b.bizType"
          class="flex items-center justify-between px-12 py-8 cursor-pointer text-13 hover:bg-gray-100"
          :class="{ 'bg-blue-50': b.bizType === selectedCode }"
          @click="select(b.bizType)"
        >
          <span class="flex items-center gap-6">
            <el-tag v-if="builtin.isBuiltin(b.bizType)" size="small" type="info" effect="plain">内置</el-tag>
            {{ b.name }} ({{ b.bizType }})
          </span>
          <el-button
            v-if="!builtin.isBuiltin(b.bizType) && !store.readOnly"
            size="small"
            link
            type="danger"
            @click.stop="removeBizType(b.bizType)"
          >删</el-button>
        </div>
        <el-empty v-if="!bizTypes.length" description="暂无" :image-size="50" />
      </div>
    </div>

    <!-- 右编辑 -->
    <div class="flex-1 overflow-y-auto p-16">
      <template v-if="current">
        <el-alert
          v-if="isCurrentBuiltin"
          title="内置业务类型,只读(不可删改)"
          type="info"
          :closable="false"
          class="mb-12"
        />
        <!-- code 醒目文字 -->
        <div class="text-20 font-bold mb-12">{{ current.bizType }}</div>
        <!-- 只读:名称 + 描述文本 -->
        <template v-if="isReadonly">
          <div class="mb-8 text-14"><span class="text-gray-500">名称:</span>{{ current.name }}</div>
          <div class="mb-12 text-14"><span class="text-gray-500">描述:</span>{{ current.description || "-" }}</div>
        </template>
        <!-- 编辑:名称 + 描述一行,填充满(描述 flex-1)-->
        <div v-else class="flex gap-12 mb-12">
          <el-input v-model="current.name" placeholder="名称" style="width: 200px" />
          <el-input v-model="current.description" placeholder="描述" class="flex-1" />
        </div>

        <!-- supportedDataTypes -->
        <div class="mt-16 mb-8 font-bold text-14 flex items-center gap-12">
          支持的数据类型
          <el-button v-if="!isReadonly" size="small" type="primary" link @click="addSupported">+ 添加</el-button>
        </div>
        <el-table ref="supportedTableRef" :data="current.supportedDataTypes" border size="small">
          <el-table-column v-if="!isReadonly" label="" width="36" align="center" key="drag">
            <template #default><span class="drag-handle cursor-move text-gray-400">⣿</span></template>
          </el-table-column>
          <el-table-column label="逻辑类型" width="160">
            <template #default="{ row }">
              <span v-if="isReadonly" class="text-13">{{ row.dataType }}</span>
              <el-select v-else v-model="row.dataType" size="small">
                <el-option v-for="dt in dataTypes" :key="dt" :label="dt" :value="dt" />
              </el-select>
            </template>
          </el-table-column>
          <el-table-column label="默认长度" width="140">
            <template #default="{ row }">
              <span v-if="isReadonly" class="text-13">{{ row.defaultLength ?? "-" }}</span>
              <el-input-number v-else v-model="row.defaultLength" size="small" :controls="false" :min="1" />
            </template>
          </el-table-column>
          <el-table-column label="默认精度" width="140">
            <template #default="{ row }">
              <span v-if="isReadonly" class="text-13">{{ row.defaultPrecision ?? "-" }}</span>
              <el-input-number v-else v-model="row.defaultPrecision" size="small" :controls="false" :min="1" />
            </template>
          </el-table-column>
          <el-table-column label="默认小数位" width="140">
            <template #default="{ row }">
              <span v-if="isReadonly" class="text-13">{{ row.defaultScale ?? "-" }}</span>
              <el-input-number v-else v-model="row.defaultScale" size="small" :controls="false" :min="0" />
            </template>
          </el-table-column>
          <el-table-column v-if="!isReadonly" label="操作" width="70" align="center">
            <template #default="{ $index }">
              <el-button size="small" link type="danger" @click="removeSupported($index)">删</el-button>
            </template>
          </el-table-column>
        </el-table>

        <!-- bizTypeData.fields(无参数隐藏表格;标题+添加按钮始终显示)-->
        <div class="mt-16 mb-8 font-bold text-14 flex items-center gap-12">
          参数配置
          <el-button v-if="!isReadonly" size="small" type="primary" link @click="addDataField">+ 添加</el-button>
        </div>
        <el-table v-show="current.bizTypeData?.fields.length" ref="dataFieldTableRef" :data="current.bizTypeData?.fields ?? []" border size="small">
            <el-table-column v-if="!isReadonly" label="" width="36" align="center" key="drag">
              <template #default><span class="drag-handle cursor-move text-gray-400">⣿</span></template>
            </el-table-column>
            <el-table-column label="参数名" width="150">
              <template #default="{ row }">
                <span v-if="isReadonly" class="text-13">{{ row.name || "-" }}</span>
                <el-input v-else v-model="row.name" size="small" />
              </template>
            </el-table-column>
            <el-table-column label="类型" width="120">
              <template #default="{ row }">
                <span v-if="isReadonly" class="text-13">{{ row.type }}</span>
                <el-select v-else v-model="row.type" size="small">
                  <el-option label="string" value="string" />
                  <el-option label="number" value="number" />
                </el-select>
              </template>
            </el-table-column>
            <el-table-column label="默认值" width="140">
              <template #default="{ row }">
                <span v-if="isReadonly" class="text-13">{{ row.default ?? "-" }}</span>
                <template v-else>
                  <el-input-number
                    v-if="row.type === 'number'"
                    v-model="row.default"
                    size="small"
                    :controls="false"
                  />
                  <el-input v-else v-model="row.default" size="small" />
                </template>
              </template>
            </el-table-column>
            <el-table-column label="描述" min-width="150">
              <template #default="{ row }">
                <span v-if="isReadonly" class="text-13">{{ row.description || "-" }}</span>
                <el-input v-else v-model="row.description" size="small" />
              </template>
            </el-table-column>
            <el-table-column label="必填" width="60" align="center">
              <template #default="{ row }">
                <span v-if="isReadonly">{{ row.required ? "✓" : "" }}</span>
                <el-checkbox v-else v-model="row.required" />
              </template>
            </el-table-column>
            <el-table-column v-if="!isReadonly" label="操作" width="70" align="center">
              <template #default="{ $index }">
                <el-button size="small" link type="danger" @click="removeDataField($index)">删</el-button>
              </template>
            </el-table-column>
          </el-table>
      </template>
      <el-empty v-else description="选择或新建业务类型" />
    </div>

    <!-- 新建业务类型弹窗:录 code + name -->
    <el-dialog v-model="newBizVisible" title="新建业务类型" width="420px" :close-on-click-modal="false">
      <el-form label-width="80px">
        <el-form-item label="code">
          <el-input v-model="newCode" placeholder="如 Date8" />
        </el-form-item>
        <el-form-item label="名称">
          <el-input v-model="newName" placeholder="如 日期8位" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="newBizVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmNewBiz">创建</el-button>
      </template>
    </el-dialog>
  </div>
  <el-empty v-else description="未打开项目" class="h-full" />
</template>
