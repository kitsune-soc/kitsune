import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  { path: '/', component: () => import('./views/MainPage.vue') },
  { path: '/about', component: () => import('./views/AboutPage.vue') },
  { path: '/messages', component: () => import('./views/MessagePage.vue') },
  {
    path: '/notifications',
    component: () => import('./views/NotificationPage.vue'),
  },
  {
    path: '/timeline',
    children: [
      {
        path: 'home',
        component: () => import('./views/timeline/HomePage.vue'),
      },
      {
        path: 'local',
        component: () => import('./views/timeline/LocalPage.vue'),
      },
      {
        path: 'federated',
        component: () => import('./views/timeline/FederatedPage.vue'),
      },
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
