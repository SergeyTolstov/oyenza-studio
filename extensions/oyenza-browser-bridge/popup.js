const button = document.querySelector("#test");
const result = document.querySelector("#result");

button.addEventListener("click", async () => {
  button.disabled = true;
  result.textContent = "Sending…";
  const reply = await chrome.runtime.sendMessage({ type: "oyenza-test" });
  result.textContent = reply?.ok ? "Sent — check the Island." : "Oyenza is not reachable. Start it and enable Browser notifications.";
  button.disabled = false;
});
