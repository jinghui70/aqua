<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import AppLayout from "@/layout/AppLayout.vue";
import { useMenuActions } from "@/composables/useMenuActions";
import { useBuiltinStore } from "@/stores/builtin";

const menu = useMenuActions();
const builtin = useBuiltinStore();
onMounted(() => {
  void builtin.load();
  menu.mount();
});
onUnmounted(() => menu.unmount());
</script>

<template>
  <AppLayout />
</template>
