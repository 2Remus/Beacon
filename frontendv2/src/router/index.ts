// @/router/index.ts
import { createRouter, createWebHashHistory } from 'vue-router';
import MainLayout from "@/views/MainView.vue";
//import AuthLayout from "@/layouts/AuthLayout.vue";

// Modular Route Imports
import { connectionsRoutes } from "../Pages/connections/connections.ts";
// import { authRoutes } from "../Pages/auth";
import { dashboardRoutes } from "../Pages/dashboard/dashboard.ts";

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