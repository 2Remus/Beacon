import backendview from './pages/backendview.vue'
import  containerview from './pages/containerview.vue'

export default [
    {
        path: '/backend',
        name: 'backend',
        component: backendview
    },
    {
        path: '/container/:id',
        name: 'containerview',
        component: containerview
    }
]