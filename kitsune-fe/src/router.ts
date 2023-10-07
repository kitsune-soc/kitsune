import { useHead } from 'unhead';
import { createRouter, createWebHistory } from 'vue-router';

import { TEMPLATE_PARAMS, TITLE_TEMPLATE } from './consts';

const routes = [
  { path: '/', component: () => import('./views/MainPage.vue') },
  {
    path: '/about',
    component: () => import('./views/AboutPage.vue'),
    meta: { title: 'About' },
  },
  {
    path: '/messages',
    component: () => import('./views/MessagePage.vue'),
    meta: { title: 'Messages' },
  },
  {
    path: '/notifications',
    component: () => import('./views/NotificationPage.vue'),
    meta: { title: 'Notifications' },
  },
  {
    path: '/timeline',
    children: [
      {
        path: 'home',
        component: () => import('./views/timeline/HomePage.vue'),
        meta: { title: 'Home' },
      },
      {
        path: 'local',
        component: () => import('./views/timeline/LocalPage.vue'),
        meta: { title: 'Local' },
      },
      {
        path: 'federated',
        component: () => import('./views/timeline/FederatedPage.vue'),
        meta: { title: 'Federated' },
      },
    ],
  },
  {
    path: '/oauth-callback',
    component: () => import('./views/OAuthCallback.vue'),
  },
  {
    path: '/posts/:id',
    component: () => import('./views/PostPage.vue'),
  },
  {
    path: '/:catchAll(.*)',
    component: () => import('./views/NotFound.vue'),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

router.afterEach((to) => {
  const title = to.meta.title;
  if (title) {
    useHead({
      title,
      titleTemplate: TITLE_TEMPLATE,
      templateParams: TEMPLATE_PARAMS,
    });
  } else {
    useHead({ title: TEMPLATE_PARAMS.siteName, titleTemplate: '%s' });
  }
});

export { router };
