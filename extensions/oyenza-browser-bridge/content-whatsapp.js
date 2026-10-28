// WhatsApp Web adapter: title/unread-count changes only. Message bodies and
// chat history intentionally never leave the page.
let previous = document.title;
new MutationObserver(() => {
  const title = document.title;
  if (title === previous) return;
  previous = title;
  const match = title.match(/\((\d+)\)/);
  if (match) {
    chrome.runtime.sendMessage({ type: "oyenza-notice", source: "whatsapp", title: "New WhatsApp message", body: `${match[1]} unread message${match[1] === "1" ? "" : "s"}` });
  }
}).observe(document.querySelector("title") || document.documentElement, { childList: true, subtree: true, characterData: true });
