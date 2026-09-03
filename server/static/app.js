"use strict";

const entriesEl = document.getElementById("entries");
const statusEl = document.getElementById("status");
const breadcrumbsEl = document.getElementById("breadcrumbs");
const upBtn = document.getElementById("up-btn");
const uploadLabel = document.getElementById("upload-label");
const uploadInput = document.getElementById("upload-input");

let currentRoot = null;
let currentPath = "";

function formatSize(bytes) {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)));
  const value = bytes / Math.pow(1024, i);
  return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

function setStatus(text) {
  if (text) {
    statusEl.textContent = text;
    statusEl.hidden = false;
  } else {
    statusEl.hidden = true;
  }
}

function renderBreadcrumbs() {
  breadcrumbsEl.innerHTML = "";

  const rootCrumb = document.createElement("button");
  rootCrumb.textContent = "Folders";
  rootCrumb.addEventListener("click", () => openRootList());
  breadcrumbsEl.appendChild(rootCrumb);

  if (!currentRoot) {
    upBtn.hidden = true;
    uploadLabel.hidden = true;
    return;
  }

  addSep();
  const rootNameCrumb = document.createElement("button");
  rootNameCrumb.textContent = currentRoot.name;
  rootNameCrumb.addEventListener("click", () => openFolder(currentRoot, ""));
  breadcrumbsEl.appendChild(rootNameCrumb);

  const segments = currentPath ? currentPath.split("/") : [];
  let acc = "";
  for (const segment of segments) {
    acc = acc ? `${acc}/${segment}` : segment;
    const pathAtSegment = acc;
    addSep();
    const crumb = document.createElement("button");
    crumb.textContent = segment;
    crumb.addEventListener("click", () => openFolder(currentRoot, pathAtSegment));
    breadcrumbsEl.appendChild(crumb);
  }

  upBtn.hidden = false;
  uploadLabel.hidden = false;

  function addSep() {
    const sep = document.createElement("span");
    sep.className = "sep";
    sep.textContent = "/";
    breadcrumbsEl.appendChild(sep);
  }
}

async function openRootList() {
  currentRoot = null;
  currentPath = "";
  renderBreadcrumbs();
  setStatus("Loading...");
  entriesEl.innerHTML = "";
  try {
    const res = await fetch("/api/folders");
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const roots = await res.json();
    if (roots.length === 0) {
      setStatus("No shared folders. Add a folder in the desktop app.");
      return;
    }
    setStatus(null);
    for (const root of roots) {
      entriesEl.appendChild(
        makeEntryEl({
          icon: "📁",
          name: root.name,
          sizeText: "",
          onClick: () => openFolder(root, ""),
        })
      );
    }
  } catch (err) {
    setStatus(`Failed to load folder list: ${err.message}`);
  }
}

async function openFolder(root, path) {
  currentRoot = root;
  currentPath = path;
  renderBreadcrumbs();
  setStatus("Loading...");
  entriesEl.innerHTML = "";
  try {
    const url = `/api/browse?root=${encodeURIComponent(root.id)}&path=${encodeURIComponent(path)}`;
    const res = await fetch(url);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const entries = await res.json();
    if (entries.length === 0) {
      setStatus("Folder is empty.");
      return;
    }
    setStatus(null);
    for (const entry of entries) {
      if (entry.is_dir) {
        entriesEl.appendChild(
          makeEntryEl({
            icon: "📁",
            name: entry.name,
            sizeText: "",
            onClick: () => openFolder(root, entry.path),
          })
        );
      } else {
        const downloadUrl = `/api/download?root=${encodeURIComponent(root.id)}&path=${encodeURIComponent(entry.path)}`;
        entriesEl.appendChild(
          makeEntryEl({
            icon: "📄",
            name: entry.name,
            sizeText: formatSize(entry.size),
            href: downloadUrl,
          })
        );
      }
    }
  } catch (err) {
    setStatus(`Failed to load folder contents: ${err.message}`);
  }
}

function makeEntryEl({ icon, name, sizeText, href, onClick }) {
  const row = document.createElement(href ? "a" : "div");
  row.className = "entry";
  if (href) {
    row.href = href;
  } else {
    row.setAttribute("role", "button");
  }
  if (onClick) {
    row.addEventListener("click", onClick);
  }

  const iconEl = document.createElement("span");
  iconEl.className = "icon";
  iconEl.textContent = icon;
  row.appendChild(iconEl);

  const nameEl = document.createElement("span");
  nameEl.className = "name";
  nameEl.textContent = name;
  row.appendChild(nameEl);

  if (sizeText) {
    const sizeEl = document.createElement("span");
    sizeEl.className = "size";
    sizeEl.textContent = sizeText;
    row.appendChild(sizeEl);
  }

  const li = document.createElement("li");
  li.appendChild(row);
  return li;
}

upBtn.addEventListener("click", () => {
  if (!currentRoot) return;
  if (!currentPath) {
    openRootList();
    return;
  }
  const parts = currentPath.split("/");
  parts.pop();
  openFolder(currentRoot, parts.join("/"));
});

uploadInput.addEventListener("change", async () => {
  if (!currentRoot || uploadInput.files.length === 0) return;
  const formData = new FormData();
  for (const file of uploadInput.files) {
    formData.append("file", file, file.name);
  }
  setStatus(`Uploading ${uploadInput.files.length} file(s)...`);
  try {
    const url = `/api/upload?root=${encodeURIComponent(currentRoot.id)}&path=${encodeURIComponent(currentPath)}`;
    const res = await fetch(url, { method: "POST", body: formData });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    await openFolder(currentRoot, currentPath);
  } catch (err) {
    setStatus(`Failed to upload file: ${err.message}`);
  } finally {
    uploadInput.value = "";
  }
});

openRootList();
