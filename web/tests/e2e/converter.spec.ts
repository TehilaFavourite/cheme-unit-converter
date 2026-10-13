import AxeBuilder from "@axe-core/playwright";
import { expect, test, type Page } from "@playwright/test";

// Labels are matched exactly: "To" must not match "factor Z".
const field = (page: Page, label: string) => page.getByLabel(label, { exact: true });
const result = (page: Page) => page.locator("#result");
const atmosphere = (page: Page) => field(page, "Local atmospheric pressure");

// Any uncaught error in the page fails the test.
test.beforeEach(({ page }) => {
  page.on("pageerror", (error) => {
    throw error;
  });
});

test("converts as you type", async ({ page }) => {
  await page.goto("/");
  await expect(field(page, "Quantity")).toHaveValue("temperature");
  await expect(field(page, "From")).toHaveValue("°C");
  await expect(field(page, "To")).toHaveValue("°F");
  await expect(result(page)).toContainText("Enter a value");

  await field(page, "Value").fill("25");
  await expect(result(page)).toContainText("25 °C =");
  await expect(page.locator(".result-number")).toHaveText("77");
  await expect(page).toHaveURL(/[?&]v=25(&|$)/);
});

test("swaps units and keeps the value", async ({ page }) => {
  await page.goto("/?v=25&from=C&to=F");
  await page.getByRole("button", { name: "Swap" }).click();
  await expect(field(page, "From")).toHaveValue("°F");
  await expect(field(page, "To")).toHaveValue("°C");
  await expect(page.locator(".result-number")).toHaveText("-3.888888889");
});

test("accepts typed aliases and follows the unit's quantity", async ({ page }) => {
  await page.goto("/?v=1");
  await field(page, "From").fill("lbmol/hr");
  await expect(field(page, "Quantity")).toHaveValue("flowRate");
  await field(page, "To").fill("kmol/h");
  await expect(page.locator(".result-number")).toHaveText("0.45359237");

  // Leaving the field shows the unit as Rust labels it.
  await field(page, "From").fill("degF");
  await field(page, "To").fill("K");
  await field(page, "To").blur();
  await expect(field(page, "From")).toHaveValue("°F");
  await expect(page.locator(".result-number")).toHaveText("255.9277778");
});

test("reveals only the context field Rust asks for, then converts", async ({ page }) => {
  await page.goto("/?v=1&from=C&to=F");
  await expect(page.locator("#context")).toBeHidden();

  await field(page, "From").fill("psig");
  await field(page, "To").fill("bar");
  await field(page, "Value").fill("30");

  await expect(atmosphere(page)).toBeVisible();
  await expect(atmosphere(page)).toHaveValue("");
  await expect(page.locator("[data-context]:not([hidden])")).toHaveCount(1);
  await expect(page.locator("#context-atm-message")).toHaveText(
    "Converting between gauge and absolute pressure requires the local atmospheric pressure",
  );
  await expect(result(page)).toHaveText("No result");

  await atmosphere(page).fill("101.325");
  await field(page, "Unit of local atmospheric pressure").selectOption("kPa");
  await expect(page.locator(".result-number")).toHaveText("3.081677188");
  await expect(page.locator("#context-atm-message")).toBeHidden();
  await expect(page).toHaveURL(/atm=101\.325kPa/);
});

test("marks filled context that the conversion does not use", async ({ page }) => {
  await page.goto("/?v=1&from=m3%2Fs&to=kg%2Fh&d=1000&mm=18");
  await expect(page.locator(".result-number")).toHaveText("3600000");
  const molarMass = page.locator('[data-context="molarMass"]');
  await expect(molarMass.locator(".note")).toBeVisible();
  await expect(molarMass.locator(".note")).toContainText("Not used by this conversion");
  await expect(page.locator('[data-context="density"] .note')).toBeHidden();

  await molarMass.getByRole("button", { name: /Clear/ }).click();
  await expect(molarMass).toBeHidden();
  await expect(page).not.toHaveURL(/mm=/);
});

test("shows errors inline next to the field responsible", async ({ page }) => {
  await page.goto("/?v=abc&from=C&to=F");
  await expect(page.locator("#value-message")).toHaveText("Invalid numeric value 'abc'");
  await expect(field(page, "Value")).toHaveAttribute("aria-invalid", "true");

  await field(page, "Value").fill("1");
  await field(page, "To").fill("kg/h");
  await expect(page.locator("#to-message")).toHaveText("Cannot convert temperature to flow rate");
  await expect(page.locator("#value-message")).toBeHidden();
});

test("restores a shared link and supports back and forward", async ({ page }) => {
  await page.goto("/?v=1000&q=flowRate&from=Sm3%2Fh&to=m3%2Fh&sf=5&rt=0C&rp=1atm&t=150C&p=5barg");
  await expect(field(page, "From")).toHaveValue("Sm³/h");
  await expect(field(page, "Standard reference temperature")).toHaveValue("0");
  await expect(field(page, "Unit of standard reference temperature")).toHaveValue("°C");
  await expect(atmosphere(page)).toBeVisible();

  await atmosphere(page).fill("1.01325");
  await field(page, "Unit of local atmospheric pressure").selectOption("bar");
  await field(page, "Gas compressibility factor Z at the actual conditions").fill("1");
  await expect(page.locator(".result-number")).toHaveText("261.04");

  await field(page, "Value").fill("2000");
  await field(page, "Value").blur();
  await expect(page.locator(".result-number")).toHaveText("522.07");
  await page.goBack();
  await expect(field(page, "Value")).toHaveValue("1000");
  await expect(page.locator(".result-number")).toHaveText("261.04");
  await page.goForward();
  await expect(field(page, "Value")).toHaveValue("2000");
});

test("ignores malformed query parameters", async ({ page }) => {
  await page.goto("/?v=25&q=nonsense&atm=kPa&zzz=1");
  await expect(field(page, "Quantity")).toHaveValue("temperature");
  await expect(page.locator("#context")).toBeHidden();
  await expect(page.locator(".result-number")).toHaveText("77");
});

test("works with the keyboard alone", async ({ page, context, browserName }) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.goto("/");

  await page.keyboard.press("Tab");
  await expect(field(page, "Value")).toBeFocused();
  await page.keyboard.type("100");

  await page.keyboard.press("Tab");
  await expect(field(page, "Quantity")).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(field(page, "From")).toBeFocused();
  await page.keyboard.press("ControlOrMeta+a");
  await page.keyboard.type("bar");
  await page.keyboard.press("Tab");
  await expect(page.getByRole("button", { name: "Swap" })).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(field(page, "To")).toBeFocused();
  await page.keyboard.press("ControlOrMeta+a");
  await page.keyboard.type("kPa");
  await expect(page.locator(".result-number")).toHaveText("10000");

  // The swap button works from the keyboard and focus is visible.
  await page.keyboard.press("Shift+Tab");
  const swap = page.getByRole("button", { name: "Swap" });
  await expect(swap).toBeFocused();
  const outline = await swap.evaluate((element) => getComputedStyle(element).outlineStyle);
  expect(outline).not.toBe("none");
  await page.keyboard.press("Enter");
  await expect(page.locator(".result-number")).toHaveText("1");

  // Reach the copy button and use it.
  const copy = page.getByRole("button", { name: "Copy result" });
  for (
    let step = 0;
    step < 6 && !(await copy.evaluate((b) => b === document.activeElement));
    step++
  ) {
    await page.keyboard.press("Tab");
  }
  await expect(copy).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.locator("#copy-status")).toHaveText("Copied");
  if (browserName === "chromium") {
    expect(await page.evaluate(() => navigator.clipboard.readText())).toBe("1");
  }
});

test("has no accessibility violations", async ({ page }) => {
  const scan = async (): Promise<void> => {
    const results = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
      .analyze();
    expect(results.passes.length).toBeGreaterThan(0);
    expect(results.violations).toEqual([]);
  };

  // Missing context revealed, with an unused filled field and the reference open.
  await page.goto("/?v=30&from=psig&to=bar&d=1000");
  await expect(atmosphere(page)).toBeVisible();
  await page.locator("summary").click();
  await scan();

  // An inline error on a main field.
  await page.goto("/?v=30&from=C&to=F&sf=x");
  await expect(page.locator("#sig-figs-message")).toBeVisible();
  await scan();

  // A result.
  await page.goto("/?v=30&from=psig&to=bar&atm=101.325kPa");
  await expect(page.locator(".result-number")).toBeVisible();
  await scan();
});
