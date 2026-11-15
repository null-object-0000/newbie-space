// Retire the old apex PWA so installed clients follow the site's move to www.
// This runs only on Cloudflare Pages; EdgeOne receives the static dist directory.
const migrationWorker = `
self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', event => event.waitUntil((async () => {
  await self.registration.unregister();
  const windows = await self.clients.matchAll({ type: 'window', includeUncontrolled: true });
  await Promise.all(windows.map(client => {
    const url = new URL(client.url);
    if (url.origin !== self.location.origin) return;
    url.protocol = 'https:';
    url.hostname = 'www.snewbie.site';
    return client.navigate(url.href);
  }));
})()));
`;
export async function onRequest(context) {
  const url = new URL(context.request.url);
  if (url.hostname === 'snewbie.site' && url.pathname === '/sw.js') {
    return new Response(migrationWorker, {
      headers: {
        'Content-Type': 'application/javascript; charset=utf-8',
        'Cache-Control': 'no-store',
      },
    });
  }
  return context.next();
}
