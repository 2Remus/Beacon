/**
 * Handles all networking and connection views
 * Base Path: /
 */
export const connectionsRoutes = [
  {
    path: 'connections', // Correct: No leading slash
    name: 'ConnectionsHome',
    component: () => import('@/Pages/connections/pages/connections.vue'),
  }
];