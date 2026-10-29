/* Hallmark · pre-emit critique: P5 H5 E4 S5 R5 V4 */
// Content and navigation work without JavaScript; copying is an enhancement.
if (navigator.clipboard && window.isSecureContext) {
  document.documentElement.classList.add("copy-enabled");
  const status = document.getElementById("copy-status");

  document.querySelectorAll("[data-copy]").forEach((button) => {
    const initialLabel = button.getAttribute("aria-label");
    let resetTimer;

    button.addEventListener("click", async () => {
      const source = document.getElementById(button.dataset.copy);
      if (!source || button.disabled) return;
      clearTimeout(resetTimer);
      button.disabled = true;
      button.dataset.state = "loading";
      button.textContent = "Copying…";
      button.setAttribute("aria-busy", "true");
      status.textContent = "";

      try {
        await navigator.clipboard.writeText(source.textContent.trim());
        button.dataset.state = "success";
        button.textContent = "Copied";
        button.setAttribute("aria-label", "Copied to clipboard");
        status.textContent = "Copied to clipboard.";
      } catch {
        button.dataset.state = "error";
        button.textContent = "Retry";
        button.setAttribute("aria-label", "Copy failed. Retry copying commands");
        status.textContent = "Clipboard access was blocked. Select and copy the commands, or retry.";
      } finally {
        button.disabled = false;
        button.removeAttribute("aria-busy");
        resetTimer = setTimeout(() => {
          button.dataset.state = "default";
          button.textContent = "Copy";
          button.setAttribute("aria-label", initialLabel);
        }, 2500);
      }
    });
  });
}
