import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router';

const routes = [
  { path: '/', component: () => import('./views/MainPage.vue') },
  { path: '/about', component: () => import('./views/AboutPage.vue') },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
