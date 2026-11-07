/*
 * Copyright (c) 2026 Costas Kirgoussios
 *
 * This software is licensed under the PolyForm Noncommercial License 1.0.0.
 * You may obtain a copy of the License at:
 * https://polyformproject.org/licenses/noncommercial/1.0.0/
 */

// The page is complete without this file: all install blocks are visible and stacked. This script
// activates interactive tabbed switching behind the OS tab bar and turns on the copy buttons.
// Zero network requests: nothing is fetched, sent, or tracked.
(function () {
  "use strict";

  var tabs = Array.prototype.slice.call(document.querySelectorAll('[role="tab"]'));
  var panels = Array.prototype.slice.call(document.querySelectorAll(".panel"));
  var tablist = document.querySelector('[role="tablist"]');
  var copyStatus = document.getElementById("copy-status");
  var statusTimers = [];

  function announce(msg) {
    if (!copyStatus) return;
    statusTimers.forEach(clearTimeout);
    copyStatus.textContent = "";
    statusTimers = [
      setTimeout(function () { copyStatus.textContent = msg; }, 50),
      setTimeout(function () { copyStatus.textContent = ""; }, 1800)
    ];
  }

  if (tabs.length && panels.length && tablist) {
    document.documentElement.classList.add("js");
    tablist.hidden = false;

    function select(tab, focus) {
      tabs.forEach(function (t) {
        var on = t === tab;
        t.setAttribute("aria-selected", on ? "true" : "false");
        t.tabIndex = on ? 0 : -1;
      });
      panels.forEach(function (p) {
        p.hidden = p.id !== tab.getAttribute("aria-controls");
      });
      if (focus) tab.focus();
    }

    tabs.forEach(function (tab, i) {
      tab.addEventListener("click", function () { select(tab, false); });
      tab.addEventListener("keydown", function (e) {
        var next = null;
        if (e.key === "ArrowRight") next = tabs[(i + 1) % tabs.length];
        else if (e.key === "ArrowLeft") next = tabs[(i + tabs.length - 1) % tabs.length];
        else if (e.key === "Home") next = tabs[0];
        else if (e.key === "End") next = tabs[tabs.length - 1];
        if (next) { e.preventDefault(); select(next, true); }
      });
    });

    // Detect user platform: Apple Silicon macOS, Intel macOS, Windows, Linux x86/arm64
    var hint = (navigator.userAgent || "") + " " + (navigator.platform || "");
    var want = "x86_64-unknown-linux-gnu";
    if (/Win/i.test(hint)) want = "x86_64-pc-windows-msvc";
    else if (/Mac/i.test(hint)) want = "aarch64-apple-darwin";
    else if (/aarch64|arm64/i.test(hint) && /Linux/i.test(hint)) want = "aarch64-unknown-linux-gnu";

    var first = tabs.filter(function (t) { return t.id === "tab-" + want; })[0] || tabs[0];
    select(first, false);
  }

  // Copy button handler for all snippet copy buttons
  Array.prototype.forEach.call(document.querySelectorAll(".copy"), function (button) {
    var targetId = button.getAttribute("data-copy");
    var pre = document.getElementById(targetId);
    if (!pre) return;

    button.hidden = false;
    var originalLabel = button.textContent;

    button.addEventListener("click", function () {
      var text = pre.textContent.replace(/^\n/, "");
      var what = button.getAttribute("aria-label") || targetId;

      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(function () {
          button.textContent = "Copied";
          button.classList.add("done");
          announce("Copied: " + what);
          setTimeout(function () {
            button.textContent = originalLabel;
            button.classList.remove("done");
          }, 1500);
        }, fallback);
      } else {
        fallback();
      }

      function fallback() {
        var range = document.createRange();
        range.selectNodeContents(pre);
        var sel = window.getSelection();
        sel.removeAllRanges();
        sel.addRange(range);
        announce("Selected for copying: press Ctrl+C");
      }
    });
  });
})();
