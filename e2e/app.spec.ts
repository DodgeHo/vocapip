import { expect, test } from "@playwright/test";

test("placement and review work after the renderer goes offline", async ({
  page,
}) => {
  await page.goto("/?view=pet");
  await expect(
    page.getByRole("button", { name: "隐藏到系统托盘" }),
  ).toBeVisible();

  if (await page.getByText("从合适的难度开始").isVisible()) {
    await page.getByRole("button", { name: /T2 基础/ }).click();
    await page.getByRole("button", { name: "使用这个水平" }).click();
  }

  const progress = page.getByRole("button", { name: "查看学习进度" });
  await progress.click();
  await expect(
    page.getByRole("dialog", { name: "查看学习进度" }),
  ).toBeVisible();
  await expect(page.getByText("今日目标")).toBeVisible();
  await progress.click();

  const pin = page.getByRole("button", { name: "将桌宠设为始终置顶" });
  await pin.click();
  await expect(
    page.getByRole("button", { name: "将桌宠恢复为普通层级" }),
  ).toHaveAttribute("aria-pressed", "true");

  const requestsWhileOffline: string[] = [];
  page.on("request", (request) => requestsWhileOffline.push(request.url()));
  await page.context().setOffline(true);

  await page
    .getByRole("button", { name: "点击 VocaPip 开始学习" })
    .click({ force: true });
  await expect(page.getByRole("button", { name: "揭晓释义" })).toBeVisible();
  await page.getByRole("button", { name: "关闭" }).click();
  await expect(page.getByRole("button", { name: "揭晓释义" })).toHaveCount(0);
  await page
    .getByRole("button", { name: "点击 VocaPip 开始学习" })
    .click({ force: true });
  await page.getByRole("button", { name: "揭晓释义" }).click();
  for (const label of ["忘记", "困难", "记得", "简单"]) {
    await expect(
      page.getByRole("button", { name: new RegExp(label) }),
    ).toBeVisible();
  }
  await page.getByRole("button", { name: /记得/ }).click();
  await expect(page.getByText(/\+\d+ XP/).first()).toBeVisible();
  expect(requestsWhileOffline).toEqual([]);
});

test("wordbook remains usable after the renderer goes offline", async ({
  page,
}) => {
  await page.goto("/?view=library");
  await expect(page.getByRole("heading", { name: "单词本" })).toBeVisible();
  await page.context().setOffline(true);
  const search = page.getByPlaceholder("搜索单词或中文释义");
  await search.fill("coherent");
  await expect(page.getByRole("heading", { name: "coherent" })).toBeVisible();
  await expect(page.locator(".word-row")).toHaveCount(1);
});

test("settings remain usable after the renderer goes offline", async ({
  page,
}) => {
  await page.goto("/?view=settings");
  await expect(page.getByRole("heading", { name: "设置" })).toBeVisible();
  await page.context().setOffline(true);
  await page.getByRole("switch", { name: "桌宠始终置顶" }).click();
  await expect(page.getByText("已立即应用")).toBeVisible();
  await expect(page.getByText("纯离线")).toBeVisible();
});
