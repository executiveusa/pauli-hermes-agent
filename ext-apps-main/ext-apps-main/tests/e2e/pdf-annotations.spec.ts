import { test, expect, type Page } from "@playwright/test";

// Increase timeout for these tests — PDF loading from arxiv can be slow
test.setTimeout(120000);

/**
 * PDF Annotation E2E Tests
 *
 * Tests the annotation capabilities of the PDF server through the basic-host UI.
 * Verifies that annotations can be added, rendered, and interacted with.
 */

/** Wait for the MCP App to load inside nested iframes. */
async function waitForAppLoad(page: Page) {
  const outerFrame = page.frameLocator("iframe").first();
  await expect(outerFrame.locator("iframe")).toBeVisible({ timeout: 30000 });
}

/** Get the app frame locator (nested: sandbox > app) */
function getAppFrame(page: Page) {
  return page.frameLocator("iframe").first().frameLocator("iframe").first();
}

/** Load the PDF server and call display_pdf with the default PDF. */
async function loadPdfServer(page: Page) {
  await page.goto("/?theme=hide");
  await expect(page.locator("select").first()).toBeEnabled({ timeout: 30000 });
  await page.locator("select").first().selectOption({ label: "PDF Server" });
  await page.click('button:has-text("Call Tool")');
  await waitForAppLoad(page);
}

/**
 * Extract the viewUUID from the display_pdf result panel.
 * The tool result is displayed as JSON in a collapsible panel.
 */
async function extractViewUUID(page: Page): Promise<string> {
  // Wait for the Tool Result panel to appear — it contains "📤 Tool Result"
  const resultPanel = page.locator('text="📤 Tool Result"').first();
  await expect(resultPanel).toBeVisible({ timeout: 30000 });

  // The result preview shows the first 100 chars including "viewUUID: ..."
  // Click to expand the result panel to see the full JSON
  await resultPanel.click();

  // Wait for the expanded result content to appear
  const resultContent = page.locator("pre").last();
  await expect(resultContent).toBeVisible({ timeout: 5000 });

  const resultText = (await resultContent.textContent()) ?? "";

  // Extract viewUUID from the JSON result
  // The text content includes: "PDF opened. viewUUID: <uuid>"
  const match = resultText.match(/viewUUID["\s:]+([a-f0-9-]{36})/);
  if (!match) {
    throw new Error(
      `Could not extract viewUUID from result: ${resultText.slice(0, 200)}`,
    );
  }
  return match[1];
}

/**
 * Call the interact tool with the given input JSON.
 * Selects the interact tool from the dropdown, fills the input, and clicks Call Tool.
 */
async function callInteract(page: Page, input: Record<string, unknown>) {
  // Select "interact" in the tool dropdown (second select on the page)
  const toolSelect = page.locator("select").nth(1);
  await toolSelect.selectOption("interact");

  // Fill the input textarea with the JSON
  const inputTextarea = page.locator("textarea");
  await inputTextarea.fill(JSON.stringify(input));

  // Click "Call Tool"
  await page.click('button:has-text("Call Tool")');
}

/** Wait for the PDF canvas to render (ensures the page is ready for annotations). */
async function waitForPdfCanvas(page: Page) {
  const appFrame = getAppFrame(page);
  await expect(appFrame.locator("canvas").first()).toBeVisible({
    timeout: 30000,
  });
  // Wait a bit for fonts and text layer to stabilize
  await page.waitForTimeout(2000);
}

test.describe("PDF Server - Annotations", () => {
  test("display_pdf result includes viewUUID and interactEnabled meta", async ({
    page,
  }) => {
    await loadPdfServer(page);

    // Wait for result to appear
    const resultPanel = page.locator('text="📤 Tool Result"').first();
    await expect(resultPanel).toBeVisible({ timeout: 30000 });

    // Expand the result panel
    await resultPanel.click();
    const resultContent = page.locator("pre").last();
    await expect(resultContent).toBeVisible({ timeout: 5000 });
    const resultText = (await resultContent.textContent()) ?? "";

    // The result should carry the viewUUID and interact flag in _meta / structuredContent
    // (interact action list lives in the tool description, not the runtime result).
    expect(resultText).toContain("viewUUID");
    expect(resultText).toContain("interactEnabled");
    expect(resultText).toContain("PDF opened");
  });

  test("interact tool is available in tool dropdown", async ({ page }) => {
    await loadPdfServer(page);

    // Verify the interact tool is available in the tool dropdown
    const toolSelect = page.locator("select").nth(1);
    const options = await toolSelect.locator("option").allTextContents();
    expect(options).toContain("interact");
  });

  test("add_annotations renders highlight on the page", async ({ page }) => {
    await loadPdfServer(page);
    await waitForPdfCanvas(page);

    const viewUUID = await extractViewUUID(page);

    // Add a highlight annotation on page 1
    await callInteract(page, {
      viewUUID,
      action: "add_annotations",
      annotations: [
        {
          id: "test-highlight-1",
          type: "highlight",
          page: 1,
          rects: [{ x: 72, y: 700, width: 300, height: 14 }],
          color: "rgba(255, 255, 0, 0.4)",
        },
      ],
    });

    // Wait for the interact result
    await page.waitForTimeout(1000);

    // Verify the annotation appears in the annotation layer inside the app frame
    const appFrame = getAppFrame(page);
    const annotationLayer = appFrame.locator("#annotation-layer");
    await expect(annotationLayer).toBeVisible({ timeout: 5000 });

    // Check that a highlight annotation element was rendered
    const highlightEl = appFrame.locator(".annotation-highlight");
    await expect(highlightEl.first()).toBeVisible({ timeout: 5000 });
  });

  test("add_annotations renders multiple annotation types", async ({
    page,
  }) => {
    await loadPdfServer(page);
    await waitForPdfCanvas(page);

    const viewUUID = await extractViewUUID(page);

    // Add multiple annotation types at once
    await callInteract(page, {
      viewUUID,
      action: "add_annotations",
      annotations: [
        {
          id: "test-highlight",
          type: "highlight",
          page: 1,
          rects: [{ x: 72, y: 700, width: 300, height: 14 }],
          color: "rgba(255, 255, 0, 0.4)",
        },
        {
          id: "test-note",
          type: "note",
          page: 1,
          x: 400,
          y: 600,
          content: "Important finding!",
          color: "#ffeb3b",
        },
        {
          id: "test-stamp",
          type: "stamp",
          page: 1,
          x: 300,
          y: 400,
          label: "APPROVED",
          color: "#4caf50",
          rotation: -15,
        },
        {
          id: "test-freetext",
          type: "freetext",
          page: 1,
          x: 100,
          y: 300,
          content: "See section 3.2",
          fontSize: 14,
          color: "#1976d2",
        },
        {
          id: "test-rect",
          type: "rectangle",
          page: 1,
          x: 50,
          y: 200,
          width: 500,
          height: 100,
          color: "#f44336",
        },
      ],
    });

    await page.waitForTimeout(1500);

    const appFrame = getAppFrame(page);

    // Verify each annotation type is rendered
    await expect(appFrame.locator(".annotation-highlight").first()).toBeVisible(
      {
        timeout: 5000,
      },
    );
    await expect(appFrame.locator(".annotation-note").first()).toBeVisible({
      timeout: 5000,
    });
    await expect(appFrame.locator(".annotation-stamp").first()).toBeVisible({
      timeout: 5000,
    });
    await expect(appFrame.locator(".annotation-freetext").first()).toBeVisible({
      timeout: 5000,
    });
    await expect(appFrame.locator(".annotation-rectangle").first()).toBeVisible(
      { timeout: 5000 },
    );
  });

  test("remove_annotations removes annotation from DOM", async ({ page }) => {
    await loadPdfServer(page);
    await waitForPdfCanvas(page);

    const viewUUID = await extractViewUUID(page);

    // Add an annotation
    await callInteract(page, {
      viewUUID,
      action: "add_annotations",
      annotations: [
        {
          id: "to-remove",
          type: "highlight",
          page: 1,
          rects: [{ x: 72, y: 700, width: 300, height: 14 }],
        },
      ],
    });

    await page.waitForTimeout(1000);

    const appFrame = getAppFrame(page);
    await expect(appFrame.locator(".annotation-highlight").first()).toBeVisible(
      {
        timeout: 5000,
      },
    );

    // Remove the annotation
    await callInteract(page, {
      viewUUID,
      action: "remove_annotations",
      ids: ["to-remove"],
    });

    await page.waitForTimeout(1000);

    // Verify the annotation is gone
    await expect(appFrame.locator(".annotation-highlight")).toHaveCount(0, {
      timeout: 5000,
    });
  });

  test("highlight_text finds and highlights text", async ({ page }) => {
    await loadPdfServer(page);
    await waitForPdfCanvas(page);

    const viewUUID = await extractViewUUID(page);

    // Use highlight_text to find and highlight "Attention" in the PDF
    await callInteract(page, {
      viewUUID,
      action: "highlight_text",
      query: "Attention",
      color: "rgba(0, 200, 255, 0.4)",
    });

    await page.waitForTimeout(2000);

    const appFrame = getAppFrame(page);
    // highlight_text creates highlight annotations, so we should see at least one
    await expect(appFrame.locator(".annotation-highlight").first()).toBeVisible(
      {
        timeout: 10000,
      },
    );
  });
});
