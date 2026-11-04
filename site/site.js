/*
 * Copyright (c) 2026 Costas Kirgoussios
 *
 * This software is licensed under the PolyForm Noncommercial License 1.0.0.
 * You may obtain a copy of the License at:
 * https://polyformproject.org/licenses/noncommercial/1.0.0/
 */

// The page is complete without this file: all five install blocks are visible and stacked. This only shows
// one block at a time behind the tab bar and turns on the copy buttons. Nothing is fetched or sent.
(function () {
  "use strict";

  var tabs = Array.prototype.slice.call(document.querySelectorAll('[role="tab"]'));
  var panels = Array.prototype.slice.call(document.querySelectorAll(".panel"));
  var tablist = document.querySelector('[role="tablist"]');
  if (!tabs.length || !panels.length || !tablist) return;

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

  // Start on the visitor's own system when the browser says so; Linux x86_64 otherwise. Apple Silicon is
  // the default Mac, since a browser cannot tell the two Mac architectures apart.
  var hint = (navigator.userAgent || "") + " " + (navigator.platform || "");
  var want = "x86_64-unknown-linux-gnu";
  if (/Win/i.test(hint)) want = "x86_64-pc-windows-msvc";
  else if (/Mac/i.test(hint)) want = "aarch64-apple-darwin";
  else if (/aarch64|arm64/i.test(hint) && /Linux/i.test(hint)) want = "aarch64-unknown-linux-gnu";
  var first = tabs.filter(function (t) { return t.id === "tab-" + want; })[0] || tabs[0];
  select(first, false);

  Array.prototype.forEach.call(document.querySelectorAll(".copy"), function (button) {
    var pre = document.getElementById(button.getAttribute("data-copy"));
    if (!pre || !navigator.clipboard) return;
    button.hidden = false;
    button.addEventListener("click", function () {
      navigator.clipboard.writeText(pre.textContent.replace(/^\n/, "")).then(function () {
        button.textContent = "Copied";
        button.classList.add("done");
        setTimeout(function () {
          button.textContent = "Copy";
          button.classList.remove("done");
        }, 1800);
      }, function () {
        button.textContent = "Select the text and copy it";
      });
    });
  });
})();
