<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import AppLayout from "@/layout/AppLayout.vue";
import { useMenuActions } from "@/composables/useMenuActions";

const menu = useMenuActions();
onMounted(() => menu.mount());
onUnmounted(() => menu.unmount());
</script>

<template>
  <AppLayout />
</template>
