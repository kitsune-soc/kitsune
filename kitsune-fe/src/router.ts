import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  { path: '/', component: () => import('./views/MainPage.vue') },
  { path: '/about', component: () => import('./views/AboutPage.vue') },
  { path: '/messages', component: () => import('./views/MainPage.vue') },
  { path: '/notifications', component: () => import('./views/MainPage.vue') },
  {
    path: '/timeline',
    children: [
      { path: 'home', component: () => import('./views/MainPage.vue') },
      { path: 'local', component: () => import('./views/MainPage.vue') },
      { path: 'federated', component: () => import('./views/MainPage.vue') },
    ],
  },
  {
    path: '/oauth-callback',
    component: () => import('./views/OAuthCallback.vue'),
  },
  {
    path: '/:catchAll(.*)',
    component: () => import('./views/NotFound.vue'),
  },
];

export const router = createRouter({
  history: createWebHistory(),
  routes,
});
