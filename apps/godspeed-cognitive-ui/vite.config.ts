import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Fixed port so the `casework-ui-up` / `-down` / `-status` recipes can manage the dev server.
// /api is proxied to the Go casework service (the UI's only backend) so the browser stays same-origin.
const api = { '/api': { target: 'http://127.0.0.1:4179', changeOrigin: false } }

export default defineConfig({
  plugins: [react()],
  server: { host: '127.0.0.1', port: 4178, strictPort: true, proxy: api },
  preview: { host: '127.0.0.1', port: 4178, strictPort: true, proxy: api },
})
