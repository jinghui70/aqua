<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import AppLayout from "@/layout/AppLayout.vue";
import { useMenuActions } from "@/composables/useMenuActions";
import { useBuiltinStore } from "@/stores/builtin";
import { useDatabaseStore } from "@/stores/database";

const menu = useMenuActions();
const builtin = useBuiltinStore();
const database = useDatabaseStore();
onMounted(() => {
  void builtin.load();
  void database.load();
  menu.mount();
});
onUnmounted(() => menu.unmount());
</script>

<template>
  <AppLayout />
</template>
