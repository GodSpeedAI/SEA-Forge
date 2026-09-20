import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

/**
 * The T03 host is a small fixture-backed surface served on a fixed, documented port so the
 * `casework-ui-up` / `-down` / `-status` recipes can manage it deterministically. strictPort keeps
 * "the URL we print" and "the URL we serve" the same thing.
 */
export default defineConfig({
  plugins: [react()],
  server: {
    host: '127.0.0.1',
    port: 4178,
    strictPort: true,
  },
  preview: {
    host: '127.0.0.1',
    port: 4178,
    strictPort: true,
  },
})
