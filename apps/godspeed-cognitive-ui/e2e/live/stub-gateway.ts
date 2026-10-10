// Tooth (1) fixture: a "gateway" that ACCEPTS intents without ever calling the kernel (it answers
// success itself) and proxies every other request to the real gateway. Run as its own process
// (the ladder drives the browser with spawnSync, which would starve an in-process server).
const listen = Number(process.env.STUB_LISTEN_PORT)
const upstream = process.env.STUB_UPSTREAM
if (!listen || !upstream) throw new Error('STUB_LISTEN_PORT and STUB_UPSTREAM are required')

Bun.serve({
  hostname: '127.0.0.1',
  port: listen,
  idleTimeout: 0,
  async fetch(req) {
    const url = new URL(req.url)
    if (req.method === 'POST' && url.pathname === '/api/intents') {
      return Response.json({ success: true, resulting_object: { id: 'case_stub0000000000' } })
    }
    const headers = new Headers(req.headers)
    headers.delete('host')
    return fetch(`http://${upstream}${url.pathname}${url.search}`, {
      method: req.method,
      headers,
      body: req.method === 'GET' || req.method === 'HEAD' ? undefined : await req.arrayBuffer(),
      redirect: 'manual',
    })
  },
})
