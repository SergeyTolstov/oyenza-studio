// WhatsApp Web adapter. It observes unread *indicators* only; contact names,
// message bodies and chat history never leave the page.
//
// WhatsApp no longer reliably updates document.title, so title changes are a
// fallback. The primary signal is the unread counter/badge in its chat list.
const unreadSelectors = [
  "[aria-label*='unread' i]",
  "[title*='unread' i]",
  "[data-testid='icon-unread-count']",
  "[data-testid*='unread']",
  "[data-icon*='unread']",
].join(",");

function visible(node) {
  return node instanceof HTMLElement && node.offsetParent !== null;
}

function numberIn(value) {
  const values = String(value || "").match(/\d+/g) || [];
  return values.reduce((total, value) => total + Number(value), 0);
}

function snapshotUnread() {
  const badges = [...document.querySelectorAll(unreadSelectors)].filter(visible);
  const badgeValues = badges.map((badge) => {
    // This value is used only locally to notice a DOM change. It is never sent.
    const localLabel = badge.getAttribute("aria-label") || badge.getAttribute("title") || badge.textContent || "";
    return { localLabel: localLabel.trim(), count: numberIn(localLabel) };
  });

  const titleCount = numberIn((document.title.match(/^\s*\((\d+)\)/) || [])[1]);
  const badgeCount = badgeValues.reduce((total, badge) => total + badge.count, 0);
  const count = Math.max(titleCount, badgeCount, badges.length ? 1 : 0);

  return {
    count,
    // A new badge with the same total count should still be noticed. This is
    // deliberately kept on the page and is not included in the notification.
    signature: `${document.title}|${badgeValues.map((badge) => badge.localLabel).join("|")}`,
  };
}

let previous = snapshotUnread();
let scheduled = false;

function checkForUnread() {
  scheduled = false;
  const current = snapshotUnread();
  const isNewUnread = current.count > 0 &&
    (current.count > previous.count || current.signature !== previous.signature);

  previous = current;
  if (!isNewUnread) return;

  chrome.runtime.sendMessage({
    type: "oyenza-notice",
    source: "whatsapp",
    title: "New WhatsApp message",
    body: `${current.count} unread message${current.count === 1 ? "" : "s"}`,
  });
}

function scheduleCheck() {
  if (scheduled) return;
  scheduled = true;
  // WhatsApp often paints the badge a moment after adding the chat row.
  window.setTimeout(checkForUnread, 250);
}

new MutationObserver(scheduleCheck).observe(document.documentElement, {
  childList: true,
  subtree: true,
  characterData: true,
  attributes: true,
  attributeFilter: ["aria-label", "title", "data-testid", "data-icon"],
});
