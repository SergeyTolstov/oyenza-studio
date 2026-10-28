# Oyenza Browser Bridge

Local Chrome/Edge extension for Island notifications from Rubin CRM and
WhatsApp Web. It posts only to `http://127.0.0.1:32148` on this PC; no cloud
relay exists.

## Install for development

1. In Oyenza Studio, open **Settings** and switch on **Browser notifications**.
2. Open `chrome://extensions` (or `edge://extensions`), turn on **Developer
   mode**, then choose **Load unpacked**.
3. Choose this folder: `extensions/oyenza-browser-bridge`.
4. Keep Oyenza Studio running, then open Rubin CRM or WhatsApp Web.

Click the extension's puzzle-piece icon and choose **Send test notification**.
If no Island card appears, first make sure Oyenza was rebuilt from this branch
and **Browser notifications** is on in Settings. Reload the extension after a
source update.

WhatsApp sends only an unread-count notification. The Rubin adapter begins
with common task-card attributes (`data-task-id`, `data-testid*="task"` and
task rows). If Rubin uses different markup or an API payload, update
`content-crm.js` after checking it in an authenticated browser session.
