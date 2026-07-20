import { createApp } from "vue";
import { createPinia } from "pinia";
import "@unocss/reset/tailwind.css";
import ElementPlus from "element-plus";
import "element-plus/dist/index.css";
import "virtual:uno.css";
import "./styles/global.css";
import App from "./App.vue";
import { router } from "./router";

createApp(App)
  .use(createPinia())
  .use(router)
  .use(ElementPlus)
  .mount("#app");
