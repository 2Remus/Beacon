/**
 * Handles all networking and connection views
 * Base Path: /
 */
export const connectRoutes = [
  {
    path: 'connect', // Correct: No leading slash
    name: 'connectHome',
    component: () => import('@/Pages/connect/pages/connect.vue'),
  }
];