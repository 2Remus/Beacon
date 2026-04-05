import { createRouter, createWebHistory } from 'vue-router'
import {DashboardRoutes} from "@/Pages/dashboard/index.js";
import MainView from "@/views/MainView.vue";
import AuthView from "@/views/AuthView.vue";
import {AuthRoutes} from "@/Pages/auth/index.js";
import {connectionsRoutes} from "@/Pages/connections/index.js";


const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      component: MainView,
      children: [
        ...DashboardRoutes // Spread the array items directly here
      ]
    },
    {
      path: '/auth',
      component: AuthView,
      children: [
          ...AuthRoutes
      ]
    },
    {
      path: '/connections',
      component: MainView,
      children: [
        ...connectionsRoutes
      ]
    }
  ]
})

export default router