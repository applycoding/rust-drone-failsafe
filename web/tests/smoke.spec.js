import { test, expect } from "@playwright/test";

test("demo loads and GPS kill leads to Land", async ({ page }) => {
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  page.on("console", (msg) => {
    if (msg.type() === "error") errors.push(msg.text());
  });

  await page.goto("/");
  await expect(page.locator("h1")).toContainText("failsafe");
  await expect
    .poll(async () => {
      const text = await page.locator("#status").textContent();
      return text && !text.includes("—") && !text.includes("Load error");
    })
    .toBe(true);

  await page.click("#cut-gcs");
  await page.click("#cut-rc");
  await page.click("#ekf-fault");
  await page.click("#drain");
  await page.click("#reset");
  await page.click("#kill-gps");

  await expect
    .poll(
      async () => {
        const text = await page.locator("#status").textContent();
        return text.includes("Land");
      },
      { timeout: 15000 }
    )
    .toBe(true);

  expect(errors).toEqual([]);
});
