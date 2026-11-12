import { createRouter, createWebHistory } from 'vue-router'
import BaseView from '../views/BaseView.vue'
import default_layout from '../layouts/default.vue' 


const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      component: default_layout,
      meta: {
        title: 'Short links'
      },
      children: [
        {
          path: '',
          name: 'home',
          component: BaseView
        },
        {
          path: '/add',
          name: 'add-url',
          component: () => import('../views/AddUrlView.vue'),
          meta: { title: 'Create link' }
        },
        {
          path: '/r/:url_code',
          name: 'manage-url',
          props:true,
          component: () => import('../views/UrlView.vue'),
          meta: { title: 'Edit link' }
        },
        {
          path: '/categories',
          name: 'category-manage',
          component: () => import('../views/CategoryManage.vue'),
          meta: { title: 'Categories' }
        },
        {
          path: '/analytical/:url_id/:url_code',
          name: 'analytical-view',
          props:true,
          component: () => import('../views/UrlVisitView.vue'),
          meta: { title: 'Link analytics' }
        },
        {
          path: '/test',
          name: 'test',
          component: () => import('../views/TextView.vue'),
          meta: { title: 'Chart preview' }
        }
      ]

    },

    
    // {
    //   path: '/about',
    //   name: 'about',
    //   component: () => import('../views/AboutView.vue'),
    // },
  ],
})

export default router
