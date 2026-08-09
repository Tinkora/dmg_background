import init, {
  create_default_document,
  export_zip,
  get_output_dimensions,
  validate_document
} from "./pkg/dmg_background_web.js";

const MAX_FILE_BYTES = 16 * 1024 * 1024;
const MAX_IMAGE_PIXELS = 16_777_216;
const SNAP_THRESHOLD = 8;

const elements = {
  app: document.querySelector("#app"),
  apply: document.querySelector("#apply-layout"),
  backgroundColor: document.querySelector("#background-color"),
  backgroundImage: document.querySelector("#background-image"),
  canvas: document.querySelector("#canvas"),
  canvasSize: document.querySelector("#canvas-size"),
  canvasWrap: document.querySelector("#canvas-wrap"),
  export: document.querySelector("#export-zip"),
  footer: document.querySelector("#footer-text"),
  formal: document.querySelector("#btn-formal"),
  item: document.querySelector("#position-item"),
  itemX: document.querySelector("#item-x"),
  itemY: document.querySelector("#item-y"),
  presets: document.querySelector("#presets"),
  preview: document.querySelector("#btn-preview"),
  reset: document.querySelector("#reset-layout"),
  status: document.querySelector("#status"),
  title: document.querySelector("#title-text"),
  volume: document.querySelector("#volume-name"),
  windowHeight: document.querySelector("#window-height"),
  windowWidth: document.querySelector("#window-width")
};

const context = elements.canvas.getContext("2d");
let backgroundImage = null;
let documentModel = null;
let dragItem = null;
let viewMode = "preview";

function setStatus(message, tone = "info") {
  elements.status.textContent = message;
  elements.status.dataset.tone = tone;
}

function errorMessage(error) {
  if (error && typeof error === "object" && "message" in error) {
    return String(error.message);
  }
  return String(error);
}

function parseInteger(input, fallback) {
  const value = Number.parseInt(input.value, 10);
  return Number.isSafeInteger(value) ? value : fallback;
}

function clamp(value, minimum, maximum) {
  return Math.max(minimum, Math.min(maximum, value));
}

function selectedItem() {
  return documentModel?.items.find((item) => item.kind === elements.item.value) ?? null;
}

function syncPositionInputs() {
  const item = selectedItem();
  if (!item || !documentModel) {
    return;
  }

  const radius = documentModel.icon_size / 2;
  elements.itemX.min = String(radius);
  elements.itemX.max = String(documentModel.window.width - radius);
  elements.itemY.min = String(radius);
  elements.itemY.max = String(documentModel.window.height - radius);
  elements.itemX.value = String(item.x);
  elements.itemY.value = String(item.y);
}

function syncCopy() {
  if (!documentModel) {
    return;
  }

  const width = documentModel.window.width;
  const title = elements.title.value.trim();
  const footer = elements.footer.value.trim();
  documentModel.texts = [];

  if (title) {
    documentModel.texts.push({
      id: "title",
      x: 0,
      y: 10,
      width,
      height: 36,
      font_size: 18,
      content: title
    });
  }
  if (footer) {
    documentModel.texts.push({
      id: "footer",
      x: 0,
      y: documentModel.window.height - 30,
      width,
      height: 20,
      font_size: 13,
      content: footer
    });
  }
}

function syncVolumeName() {
  if (!documentModel) {
    return;
  }

  const volumeName = elements.volume.value.trim() || "Untitled";
  documentModel.export.volume_name = volumeName;
  const application = documentModel.items.find((item) => item.kind === "application");
  if (application) {
    application.label = `${volumeName}.app`;
  }
}

function applyPositionInputs(target) {
  if (!target || !documentModel) {
    return;
  }

  const radius = documentModel.icon_size / 2;
  target.x = clamp(
    parseInteger(elements.itemX, target.x),
    radius,
    documentModel.window.width - radius
  );
  target.y = clamp(
    parseInteger(elements.itemY, target.y),
    radius,
    documentModel.window.height - radius
  );
}

function buildDocument(statusMessage = "Layout updated") {
  const width = parseInteger(elements.windowWidth, 800);
  const height = parseInteger(elements.windowHeight, 450);
  const volumeName = elements.volume.value.trim() || "Untitled";
  const previous = documentModel;
  const selectedKind = elements.item.value;
  const requestedX = parseInteger(elements.itemX, null);
  const requestedY = parseInteger(elements.itemY, null);

  try {
    const next = create_default_document(width, height, volumeName);
    const radius = next.icon_size / 2;

    for (const item of next.items) {
      const previousItem = previous?.items.find((candidate) => candidate.kind === item.kind);
      if (previousItem) {
        item.x = clamp(previousItem.x, radius, width - radius);
        item.y = clamp(previousItem.y, radius, height - radius);
      }
      if (item.kind === selectedKind) {
        if (requestedX !== null) {
          item.x = clamp(requestedX, radius, width - radius);
        }
        if (requestedY !== null) {
          item.y = clamp(requestedY, radius, height - radius);
        }
      }
    }

    documentModel = next;
    syncCopy();
    documentModel = validate_document(documentModel);
    syncPositionInputs();
    render();
    setStatus(statusMessage);
  } catch (error) {
    setStatus(errorMessage(error), "error");
  }
}

function setView(mode) {
  viewMode = mode;
  const previewActive = mode === "preview";
  elements.preview.setAttribute("aria-pressed", String(previewActive));
  elements.formal.setAttribute("aria-pressed", String(!previewActive));
  elements.canvasWrap.dataset.mode = mode;
  render();
}

function drawBackground(target) {
  const { width, height } = documentModel.window;
  if (backgroundImage) {
    target.drawImage(backgroundImage, 0, 0, width, height);
    return;
  }
  target.fillStyle = elements.backgroundColor.value;
  target.fillRect(0, 0, width, height);
}

function drawArrow(target) {
  const application = documentModel.items.find((item) => item.kind === "application");
  const applications = documentModel.items.find(
    (item) => item.kind === "applications_alias"
  );
  if (!application || !applications) {
    return;
  }

  const radius = documentModel.icon_size / 2;
  const fromX = application.x + radius;
  const fromY = application.y;
  const toX = applications.x - radius;
  const toY = applications.y;
  target.strokeStyle = "#5b625f";
  target.lineWidth = 3;
  target.setLineDash([8, 5]);
  target.beginPath();
  target.moveTo(fromX, fromY);
  target.lineTo(toX, toY);
  target.stroke();
  target.setLineDash([]);

  const deltaX = toX - fromX;
  const deltaY = toY - fromY;
  const length = Math.hypot(deltaX, deltaY);
  if (length === 0) {
    return;
  }
  const unitX = deltaX / length;
  const unitY = deltaY / length;
  target.fillStyle = "#5b625f";
  target.beginPath();
  target.moveTo(toX, toY);
  target.lineTo(toX - unitX * 13 - unitY * 7, toY - unitY * 13 + unitX * 7);
  target.lineTo(toX - unitX * 13 + unitY * 7, toY - unitY * 13 - unitX * 7);
  target.closePath();
  target.fill();
}

function drawFittedText(target, text, x, y, maximumWidth, initialSize, weight = "400") {
  let size = initialSize;
  do {
    target.font = `${weight} ${size}px system-ui`;
    if (target.measureText(text).width <= maximumWidth) {
      break;
    }
    size -= 1;
  } while (size > 10);
  target.fillText(text, x, y, maximumWidth);
}

function drawCopy(target) {
  const width = documentModel.window.width;
  target.fillStyle = "#26302c";
  target.textAlign = "center";
  target.textBaseline = "alphabetic";
  const title = elements.title.value.trim();
  const footer = elements.footer.value.trim();
  if (title) {
    drawFittedText(target, title, width / 2, 40, width - 48, 18, "700");
  }
  if (footer) {
    drawFittedText(
      target,
      footer,
      width / 2,
      documentModel.window.height - 16,
      width - 48,
      13
    );
  }
}

function roundedRectangle(target, x, y, width, height, radius) {
  target.beginPath();
  target.roundRect(x, y, width, height, radius);
}

function drawItems(target) {
  for (const item of documentModel.items) {
    const radius = documentModel.icon_size / 2;
    target.fillStyle = item.kind === "application" ? "#2879bf" : "#39865d";
    roundedRectangle(
      target,
      item.x - radius,
      item.y - radius,
      documentModel.icon_size,
      documentModel.icon_size,
      12
    );
    target.fill();
    target.fillStyle = "#ffffff";
    target.textAlign = "center";
    target.textBaseline = "middle";
    drawFittedText(target, item.label, item.x, item.y, documentModel.icon_size - 14, 13, "650");
  }
}

function snapPosition(x, y, activeItem) {
  const { width, height } = documentModel.window;
  const xCandidates = [width / 3, width / 2, (width * 2) / 3];
  const yCandidates = [height / 3, height / 2, (height * 2) / 3];
  for (const item of documentModel.items) {
    if (item !== activeItem) {
      xCandidates.push(item.x);
      yCandidates.push(item.y);
    }
  }

  let snappedX = x;
  let snappedY = y;
  for (const candidate of xCandidates) {
    if (Math.abs(candidate - x) < SNAP_THRESHOLD) {
      snappedX = Math.round(candidate);
      break;
    }
  }
  for (const candidate of yCandidates) {
    if (Math.abs(candidate - y) < SNAP_THRESHOLD) {
      snappedY = Math.round(candidate);
      break;
    }
  }

  const guides = [];
  if (snappedX !== x) {
    guides.push({ axis: "vertical", position: snappedX });
  }
  if (snappedY !== y) {
    guides.push({ axis: "horizontal", position: snappedY });
  }
  return { x: snappedX, y: snappedY, guides };
}

function drawGuides(target) {
  if (!dragItem) {
    return;
  }
  target.strokeStyle = "rgb(24 119 94 / 65%)";
  target.lineWidth = 1;
  target.setLineDash([4, 4]);
  for (const guide of documentModel.guides) {
    target.beginPath();
    if (guide.axis === "vertical") {
      target.moveTo(guide.position, 0);
      target.lineTo(guide.position, documentModel.window.height);
    } else {
      target.moveTo(0, guide.position);
      target.lineTo(documentModel.window.width, guide.position);
    }
    target.stroke();
  }
  target.setLineDash([]);
}

function drawScene(target, includeItems) {
  drawBackground(target);
  drawArrow(target);
  drawCopy(target);
  if (includeItems) {
    drawItems(target);
  }
}

function render() {
  if (!documentModel) {
    return;
  }
  const ratio = Math.min(window.devicePixelRatio || 1, 2);
  const { width, height } = documentModel.window;
  elements.canvas.width = Math.round(width * ratio);
  elements.canvas.height = Math.round(height * ratio);
  elements.canvas.style.width = `${width}px`;
  elements.canvas.style.height = `${height}px`;
  elements.canvasSize.textContent = `${width} x ${height}`;
  context.setTransform(ratio, 0, 0, ratio, 0, 0);
  drawScene(context, viewMode === "preview");
  drawGuides(context);
}

function canvasPoint(event) {
  const bounds = elements.canvas.getBoundingClientRect();
  return {
    x: ((event.clientX - bounds.left) * documentModel.window.width) / bounds.width,
    y: ((event.clientY - bounds.top) * documentModel.window.height) / bounds.height
  };
}

function hitTest(point) {
  const radius = documentModel.icon_size / 2;
  return (
    documentModel.items.find(
      (item) =>
        point.x >= item.x - radius &&
        point.x <= item.x + radius &&
        point.y >= item.y - radius &&
        point.y <= item.y + radius
    ) ?? null
  );
}

function moveSelectedItem(deltaX, deltaY) {
  const item = selectedItem();
  if (!item || !documentModel) {
    return;
  }
  const radius = documentModel.icon_size / 2;
  item.x = clamp(item.x + deltaX, radius, documentModel.window.width - radius);
  item.y = clamp(item.y + deltaY, radius, documentModel.window.height - radius);
  documentModel.guides = [];
  syncPositionInputs();
  render();
  setStatus("Layout updated");
}

function canvasBlob(canvas) {
  return new Promise((resolve, reject) => {
    canvas.toBlob((blob) => {
      if (blob) {
        resolve(blob);
      } else {
        reject(new Error("Canvas PNG encoding failed"));
      }
    }, "image/png");
  });
}

async function renderExport(width, height, scale, includeItems) {
  const canvas = document.createElement("canvas");
  canvas.width = width * scale;
  canvas.height = height * scale;
  const target = canvas.getContext("2d");
  target.scale(scale, scale);
  drawScene(target, includeItems);
  return new Uint8Array(await (await canvasBlob(canvas)).arrayBuffer());
}

async function exportZip() {
  if (!documentModel) {
    return;
  }
  elements.export.disabled = true;
  setStatus("Exporting");

  try {
    syncCopy();
    syncVolumeName();
    documentModel = validate_document(documentModel);
    const dimensions = get_output_dimensions(documentModel);
    const oneX = await renderExport(
      dimensions.one_x.width,
      dimensions.one_x.height,
      1,
      false
    );
    const twoX = await renderExport(
      dimensions.one_x.width,
      dimensions.one_x.height,
      2,
      false
    );
    const preview = await renderExport(
      dimensions.one_x.width,
      dimensions.one_x.height,
      1,
      true
    );
    const zipBytes = export_zip(documentModel, oneX, twoX, preview);
    const blob = new Blob([zipBytes], { type: "application/zip" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `${documentModel.export.volume_name}-background.zip`;
    document.body.append(link);
    link.click();
    link.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 0);
    setStatus("ZIP downloaded");
  } catch (error) {
    setStatus(`Export failed: ${errorMessage(error)}`, "error");
  } finally {
    elements.export.disabled = false;
  }
}

elements.preview.addEventListener("click", () => setView("preview"));
elements.formal.addEventListener("click", () => setView("formal"));
elements.apply.addEventListener("click", () => buildDocument());
elements.export.addEventListener("click", exportZip);
elements.item.addEventListener("change", syncPositionInputs);

elements.presets.addEventListener("click", (event) => {
  const button = event.target.closest("button[data-width][data-height]");
  if (!button) {
    return;
  }
  elements.windowWidth.value = button.dataset.width;
  elements.windowHeight.value = button.dataset.height;
  buildDocument("Preset applied");
});

for (const input of [elements.title, elements.footer]) {
  input.addEventListener("input", () => {
    syncCopy();
    render();
    setStatus("Layout updated");
  });
}

elements.backgroundColor.addEventListener("input", () => {
  render();
  setStatus("Background updated");
});

elements.backgroundImage.addEventListener("change", async () => {
  const file = elements.backgroundImage.files?.[0];
  if (!file) {
    backgroundImage?.close();
    backgroundImage = null;
    render();
    setStatus("Background cleared");
    return;
  }
  if (file.size > MAX_FILE_BYTES) {
    elements.backgroundImage.value = "";
    setStatus("File exceeds the 16 MiB limit.", "error");
    return;
  }
  if (!["image/png", "image/jpeg"].includes(file.type)) {
    elements.backgroundImage.value = "";
    setStatus("Choose a PNG or JPEG file.", "error");
    return;
  }

  try {
    const decoded = await createImageBitmap(file);
    if (decoded.width * decoded.height > MAX_IMAGE_PIXELS) {
      decoded.close();
      elements.backgroundImage.value = "";
      setStatus("Decoded image exceeds the 16 megapixel limit.", "error");
      return;
    }
    backgroundImage?.close();
    backgroundImage = decoded;
    render();
    setStatus("Background loaded");
  } catch {
    elements.backgroundImage.value = "";
    setStatus("The browser could not decode this image.", "error");
  }
});

elements.canvas.addEventListener("pointerdown", (event) => {
  if (!documentModel) {
    return;
  }
  const item = hitTest(canvasPoint(event));
  if (!item) {
    return;
  }
  dragItem = item;
  elements.item.value = item.kind;
  syncPositionInputs();
  elements.canvas.setPointerCapture(event.pointerId);
  event.preventDefault();
});

elements.canvas.addEventListener("pointermove", (event) => {
  if (!dragItem || !documentModel) {
    return;
  }
  const radius = documentModel.icon_size / 2;
  const point = canvasPoint(event);
  const snapped = snapPosition(Math.round(point.x), Math.round(point.y), dragItem);
  dragItem.x = clamp(snapped.x, radius, documentModel.window.width - radius);
  dragItem.y = clamp(snapped.y, radius, documentModel.window.height - radius);
  documentModel.guides = snapped.guides;
  syncPositionInputs();
  render();
});

function finishDrag(event) {
  if (!dragItem) {
    return;
  }
  if (elements.canvas.hasPointerCapture(event.pointerId)) {
    elements.canvas.releasePointerCapture(event.pointerId);
  }
  dragItem = null;
  render();
  setStatus("Layout updated");
}

elements.canvas.addEventListener("pointerup", finishDrag);
elements.canvas.addEventListener("pointercancel", finishDrag);

elements.canvas.addEventListener("keydown", (event) => {
  const step = event.shiftKey ? 10 : 1;
  const moves = {
    ArrowDown: [0, step],
    ArrowLeft: [-step, 0],
    ArrowRight: [step, 0],
    ArrowUp: [0, -step]
  };
  const move = moves[event.key];
  if (!move) {
    return;
  }
  event.preventDefault();
  moveSelectedItem(move[0], move[1]);
});

elements.reset.addEventListener("click", () => {
  backgroundImage?.close();
  backgroundImage = null;
  elements.backgroundImage.value = "";
  elements.backgroundColor.value = "#f1f3f2";
  elements.windowWidth.value = "800";
  elements.windowHeight.value = "450";
  elements.volume.value = "Example";
  elements.title.value = "Drag Example to Applications";
  elements.footer.value = "Find the app in Applications after installation";
  documentModel = null;
  elements.itemX.value = "";
  elements.itemY.value = "";
  buildDocument("Layout reset");
});

try {
  await init();
  buildDocument("Ready");
  elements.app.setAttribute("aria-busy", "false");
  elements.apply.disabled = false;
  elements.export.disabled = false;
  elements.reset.disabled = false;
} catch (error) {
  elements.app.setAttribute("aria-busy", "false");
  setStatus(`Initialization failed: ${errorMessage(error)}`, "error");
  console.error(error);
}
