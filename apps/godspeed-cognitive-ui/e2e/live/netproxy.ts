export {}
// A transparent pass-through between a browser and the real gateway, run as its own process so the
// recovery journey can sever the network (kill -9: every open SSE stream drops at once) and restore
// it, without touching the gateway, its sessions or the kernel. Same-origin: the browser talks to
// this port, which forwards byte-for-byte (Host/Origin/Referer are rewritten to the upstream's).
const listen = Number(process.env.PROXY_LISTEN_PORT)
const upstream = process.env.PROXY_UPSTREAM
if (!listen || !upstream) throw new Error('PROXY_LISTEN_PORT and PROXY_UPSTREAM are required')

Bun.serve({
  hostname: '127.0.0.1',
  port: listen,
  idleTimeout: 0,
  async fetch(req) {
    const url = new URL(req.url)
    const headers = new Headers(req.headers)
    headers.delete('host')
    headers.delete('accept-encoding')
    if (headers.has('origin')) headers.set('origin', `http://${upstream}`)
    if (headers.has('referer')) headers.set('referer', `http://${upstream}${new URL(headers.get('referer')!).pathname}`)
    const res = await fetch(`http://${upstream}${url.pathname}${url.search}`, {
      method: req.method,
      headers,
      body: req.method === 'GET' || req.method === 'HEAD' ? undefined : await req.arrayBuffer(),
      redirect: 'manual',
    })
    return res
  },
})
