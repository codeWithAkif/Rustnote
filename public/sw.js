// Çevrimdışı çalışma: önce ağ, olmazsa önbellek.
const C = "rustnote-v1";
self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (e) => e.waitUntil(self.clients.claim()));
self.addEventListener("fetch", (e) => {
  if (e.request.method !== "GET") return;
  e.respondWith(
    fetch(e.request)
      .then((r) => { const c = r.clone(); caches.open(C).then((x) => x.put(e.request, c)); return r; })
      .catch(() => caches.match(e.request))
  );
});
