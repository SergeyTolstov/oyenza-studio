// Rubin CRM adapter. It deliberately sends only a task label, never cookies,
// page HTML, credentials or other CRM data. Initial cards are remembered;
// only cards inserted afterwards become Island notifications.
const known = new Set();
let ready = false;

function taskNodes(root = document) {
  return [...root.querySelectorAll("[data-task-id], [data-testid*='task' i], [data-id][role='row']")];
}

function taskLabel(node) {
  return (node.getAttribute("data-task-title") || node.innerText || "").split("\n").map(s => s.trim()).find(Boolean)?.slice(0, 120) || "";
}

function taskKey(node, label) {
  return node.getAttribute("data-task-id") || node.getAttribute("data-id") || label;
}

function see(node) {
  const label = taskLabel(node);
  const key = taskKey(node, label);
  if (!key || !label || known.has(key)) return;
  known.add(key);
  if (ready) chrome.runtime.sendMessage({ type: "oyenza-notice", source: "crm", title: "New task", body: label });
}

function scan(root) { taskNodes(root).forEach(see); }
scan();
ready = true;
new MutationObserver((mutations) => {
  for (const mutation of mutations) {
    for (const node of mutation.addedNodes) {
      if (node.nodeType !== Node.ELEMENT_NODE) continue;
      see(node);
      scan(node);
    }
  }
}).observe(document.documentElement, { childList: true, subtree: true });
