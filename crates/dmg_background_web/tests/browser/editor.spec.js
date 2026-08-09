import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";

import { expect, test } from "@playwright/test";

function zipMember(archivePath, member) {
  return execFileSync("unzip", ["-p", archivePath, member], {
    encoding: null,
    maxBuffer: 32 * 1024 * 1024
  });
}

function pngDimensions(bytes) {
  expect(bytes.subarray(0, 8)).toEqual(
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])
  );
  return {
    width: bytes.readUInt32BE(16),
    height: bytes.readUInt32BE(20)
  };
}

test("returns plain objects and stable resource errors from real WASM", async ({ page }) => {
  await page.goto("/");

  const result = await page.evaluate(async () => {
    const wasm = await import("/pkg/dmg_background_web.js");
    await wasm.default();

    const document = wasm.create_default_document(800, 450, "Tinkora");
    const dimensions = wasm.get_output_dimensions(document);
    let error = null;
    try {
      wasm.create_default_document(4096, 1025, "Too large");
    } catch (value) {
      error = { code: value.code, message: value.message };
    }
    const oversized = structuredClone(document);
    oversized.window = { width: 4096, height: 1025 };
    let dimensionsError = null;
    try {
      wasm.get_output_dimensions(oversized);
    } catch (value) {
      dimensionsError = { code: value.code, message: value.message };
    }

    return {
      constructor: document.constructor.name,
      schema: document.$schema,
      window: document.window,
      dimensions,
      error,
      dimensionsError
    };
  });

  expect(result).toEqual({
    constructor: "Object",
    schema: "https://tinkora.github.io/dmg_background/schema/dmg-layout-v1.json",
    window: { width: 800, height: 450 },
    dimensions: {
      one_x: { width: 800, height: 450 },
      two_x: { width: 1600, height: 900 }
    },
    error: {
      code: "OUTPUT_PIXEL_BUDGET_EXCEEDED",
      message: "Retina output requires 16793600 pixels, above the 16777216 pixel budget"
    },
    dimensionsError: {
      code: "OUTPUT_PIXEL_BUDGET_EXCEEDED",
      message: "Retina output requires 16793600 pixels, above the 16777216 pixel budget"
    }
  });
});

test("supports labelled controls and keyboard positioning", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByText("Ready", { exact: true })).toBeVisible();

  await page.getByLabel("Volume name").fill("Tinkora");
  await page.getByLabel("Title").fill("Drag Tinkora to Applications");
  await page.getByLabel("Item to position").selectOption("application");
  await page.getByLabel("Item X").fill("280");
  await page.getByLabel("Item Y").fill("220");
  await page.getByRole("button", { name: "Apply layout" }).click();

  await expect(page.getByRole("status")).toHaveText("Layout updated");
  await expect(page.getByRole("button", { name: "Finder preview" })).toHaveAttribute(
    "aria-pressed",
    "true"
  );
  await expect(page.getByRole("button", { name: "Formal background" })).toHaveAttribute(
    "aria-pressed",
    "false"
  );

  const canvas = page.getByRole("img", { name: "DMG layout preview" });
  await canvas.focus();
  await expect(canvas).toBeFocused();
  await page.keyboard.press("ArrowRight");
  await expect(page.getByLabel("Item X")).toHaveValue("281");
});

test("rejects an oversized background before decoding", async ({ page }) => {
  await page.goto("/");

  await page.getByLabel("Background image").setInputFiles({
    name: "oversized.png",
    mimeType: "image/png",
    buffer: Buffer.alloc(16 * 1024 * 1024 + 1)
  });

  await expect(page.getByRole("status")).toHaveText("File exceeds the 16 MiB limit.");
});

test("exports the exact ZIP contract with expected PNG dimensions", async ({ page }) => {
  await page.goto("/");
  await page.getByLabel("Volume name").fill("Tinkora");
  await page.getByLabel("Title").fill("Drag Tinkora to Applications");

  const downloadPromise = page.waitForEvent("download");
  await page.getByRole("button", { name: "Export ZIP" }).click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toBe("Tinkora-background.zip");

  const archivePath = await download.path();
  expect(archivePath).not.toBeNull();
  expect((await readFile(archivePath)).subarray(0, 4)).toEqual(Buffer.from("PK\x03\x04", "binary"));

  const members = execFileSync("unzip", ["-Z1", archivePath], { encoding: "utf8" })
    .trim()
    .split("\n")
    .sort();
  expect(members).toEqual(
    [
      ".background/background.png",
      ".background/background@2x.png",
      "README.txt",
      "dmg_layout.json",
      "preview.png"
    ].sort()
  );

  const layout = JSON.parse(zipMember(archivePath, "dmg_layout.json").toString("utf8"));
  expect(layout.$schema).toBe(
    "https://tinkora.github.io/dmg_background/schema/dmg-layout-v1.json"
  );
  expect(layout.export.volume_name).toBe("Tinkora");
  expect(layout.items.find((item) => item.kind === "application").label).toBe("Tinkora.app");
  expect(layout.texts.find((text) => text.id === "title").content).toBe(
    "Drag Tinkora to Applications"
  );

  const oneX = zipMember(archivePath, ".background/background.png");
  const twoX = zipMember(archivePath, ".background/background@2x.png");
  const preview = zipMember(archivePath, "preview.png");
  expect(pngDimensions(oneX)).toEqual({ width: 800, height: 450 });
  expect(pngDimensions(twoX)).toEqual({ width: 1600, height: 900 });
  expect(pngDimensions(preview)).toEqual({ width: 800, height: 450 });
  expect(preview.equals(oneX)).toBe(false);
});

test("loads without external requests, console problems, or horizontal overflow", async ({
  baseURL,
  page
}) => {
  const problems = [];
  const externalRequests = [];
  const failedResponses = [];
  const expectedOrigin = new URL(baseURL).origin;

  page.on("console", (message) => {
    if (["error", "warning"].includes(message.type())) {
      problems.push(`${message.type()}: ${message.text()}`);
    }
  });
  page.on("pageerror", (error) => problems.push(`pageerror: ${error.message}`));
  page.on("request", (request) => {
    if (new URL(request.url()).origin !== expectedOrigin) {
      externalRequests.push(request.url());
    }
  });
  page.on("response", (response) => {
    if (response.status() >= 400) {
      failedResponses.push(`${response.status()} ${response.url()}`);
    }
  });

  await page.goto("/");
  await page.waitForLoadState("networkidle");
  await expect(page.getByRole("heading", { name: "DMG Background" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Export ZIP" })).toBeEnabled();

  const layout = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth
  }));
  expect(layout.scrollWidth).toBe(layout.clientWidth);
  expect(externalRequests).toEqual([]);
  expect(failedResponses).toEqual([]);
  expect(problems).toEqual([]);
});
