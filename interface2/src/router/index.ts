// @/router/index.ts
import { createRouter, createWebHashHistory } from 'vue-router';
import MainLayout from "@/views/MainView.vue";
//import AuthLayout from "@/layouts/AuthLayout.vue";

// Modular Route Imports
// @ts-ignore
import { connectionsRoutes } from "@/Pages/connections/connections";
// import { authRoutes } from "../Pages/auth";
// @ts-ignore
import { dashboardRoutes } from "@/Pages/dashboard/dashboard";
//import {connectRoutes} from '@/Pages/connect/connect.ts'

const router = createRouter({
  // Electron usually fails on refresh with WebHistory; HashHistory is safer
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      component: MainLayout,
      children: [
        ...connectionsRoutes,
        ...dashboardRoutes,
        //...connectRoutes
      ]
    },
    // {
    //   path: '/auth',
    //   component: AuthLayout,
    //   children: authRoutes
    // },
    // Fallback/404 can go here
    {
      path: '/:pathMatch(.*)*',
      redirect: '/'
    }
  ]
});

export default router;