/**
 * Handles server monitoring and dashboard views
 * Base Path: /connections (as per your main router config)
 */
export const dashboardRoutes = [
  {
    path: '', // Correct: This matches "/"
    name: 'DashboardOverview',
    component: () => import('@/Pages/dashboard/pages/dashboard.vue'),
  }
];