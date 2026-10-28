const endpoint = "http://127.0.0.1:32148/v1/notice";
const recentlySent = new Map();

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message?.type === "oyenza-test") {
    postNotice("browser", "Test notification", "The local browser bridge is connected.").then(
      () => sendResponse({ ok: true }),
      () => sendResponse({ ok: false })
    );
    return true;
  }
  if (message?.type !== "oyenza-notice") return;
  const source = String(message.source || "").slice(0, 24);
  const title = String(message.title || "").trim().slice(0, 120);
  const body = String(message.body || "").trim().slice(0, 220);
  if (!source || !title) return;
  const key = `${source}\n${title}\n${body}`;
  const now = Date.now();
  if (recentlySent.get(key) > now - 15_000) return;
  recentlySent.set(key, now);
  for (const [old, at] of recentlySent) if (at < now - 60_000) recentlySent.delete(old);
  postNotice(source, title, body).catch(() => {
    // Oyenza may be closed. The extension remains quiet and retries on the
    // next real browser event rather than polling localhost.
  });
});

function postNotice(source, title, body) {
  return fetch(endpoint, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ source, title, body })
  });
}
