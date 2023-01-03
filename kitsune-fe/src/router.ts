import { createRouter, createWebHashHistory } from 'vue-router';

const routes = [
  { path: '/', component: () => import('./views/MainPage.vue') },
  { path: '/home', component: () => import('./views/MainPage.vue') },
  { path: '/notifications', component: () => import('./views/MainPage.vue') },
  { path: '/messages', component: () => import('./views/MainPage.vue') },
  { path: '/local', component: () => import('./views/MainPage.vue') },
  { path: '/federated', component: () => import('./views/MainPage.vue') },
  { path: '/about', component: () => import('./views/AboutPage.vue') },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
