// Telegram Web adapter. It reads unread indicators only: neither message
// bodies nor contact names are sent to Oyenza Studio.
const selectors = ["[aria-label*='unread' i]", "[data-testid*='unread']", "[data-icon*='unread']"].join(",");

function snapshot() {
  const badges = [...document.querySelectorAll(selectors)].filter((node) => node instanceof HTMLElement && node.offsetParent !== null);
  const title = document.title.match(/^\s*\((\d+)\)/);
  const count = Math.max(Number(title?.[1] || 0), badges.length ? 1 : 0);
  return { count, signature: `${document.title}|${badges.map((node) => node.getAttribute("aria-label") || node.textContent || "").join("|")}` };
}

let previous = snapshot();
let scheduled = false;
function check() {
  scheduled = false;
  const current = snapshot();
  const changed = current.count > 0 && (current.count > previous.count || current.signature !== previous.signature);
  previous = current;
  if (changed) chrome.runtime.sendMessage({ type: "oyenza-notice", source: "telegram", title: "New Telegram message", body: `${current.count} unread message${current.count === 1 ? "" : "s"}` });
}
new MutationObserver(() => {
  if (!scheduled) { scheduled = true; window.setTimeout(check, 250); }
}).observe(document.documentElement, { childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ["aria-label", "title", "data-testid", "data-icon"] });
