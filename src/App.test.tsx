// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { getView } from "./App";
import { PetCharacter } from "./components/PetCharacter";
import { LibraryView } from "./views/LibraryView";
import { PetView } from "./views/PetView";
import { SettingsView } from "./views/SettingsView";
afterEach(() => cleanup());
beforeEach(() => localStorage.clear());
describe("view routing", () => {
  it("selects each supported URL and falls back to pet", () => {
    expect(getView("?view=library")).toBe("library");
    expect(getView("?view=settings")).toBe("settings");
    expect(getView("?view=unknown")).toBe("pet");
  });
});
describe("pet learning flow", () => {
  it("completes self placement, reveals a card, and exposes four ratings", async () => {
    const user = userEvent.setup();
    render(<PetView />);
    expect(
      await screen.findByRole("button", { name: "隐藏到系统托盘" }),
    ).toBeTruthy();
    expect(await screen.findByText("从合适的难度开始")).toBeTruthy();
    await user.click(screen.getByText("T3 进阶"));
    await user.click(screen.getByRole("button", { name: "使用这个水平" }));
    await waitFor(() =>
      expect(screen.queryByText("从合适的难度开始")).toBeNull(),
    );
    await user.click(
      screen.getByRole("button", { name: "点击 VocaPip 开始学习" }),
    );
    expect(
      await screen.findByRole("heading", { level: 1, name: /steady|notice/ }),
    ).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "揭晓释义" }));
    for (const label of ["忘记", "困难", "记得", "简单"])
      expect(
        screen.getByRole("button", { name: new RegExp(label) }),
      ).toBeTruthy();
    await user.click(screen.getByRole("button", { name: /记得/ }));
    expect((await screen.findAllByText(/\+8 XP/)).length).toBeGreaterThan(0);
  });

  it("keeps progress, close, and pin controls attached to the pet", async () => {
    const user = userEvent.setup();
    render(<PetView />);
    await screen.findByText("从合适的难度开始");
    await user.click(screen.getByText("T2 基础"));
    await user.click(screen.getByRole("button", { name: "使用这个水平" }));
    await waitFor(() =>
      expect(screen.queryByText("从合适的难度开始")).toBeNull(),
    );

    const progress = screen.getByRole("button", { name: "查看学习进度" });
    await user.click(progress);
    expect(
      await screen.findByRole("dialog", { name: "查看学习进度" }),
    ).toBeTruthy();
    expect(screen.getByText("今日目标")).toBeTruthy();
    await user.click(progress);
    expect(screen.queryByRole("dialog", { name: "查看学习进度" })).toBeNull();

    const pin = screen.getByRole("button", { name: "将桌宠设为始终置顶" });
    await user.click(pin);
    expect(
      screen
        .getByRole("button", { name: "将桌宠恢复为普通层级" })
        .getAttribute("aria-pressed"),
    ).toBe("true");

    await user.click(
      screen.getByRole("button", { name: "点击 VocaPip 开始学习" }),
    );
    expect(await screen.findByRole("heading", { level: 1 })).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "关闭" }));
    expect(screen.queryByRole("heading", { level: 1 })).toBeNull();
  });
});
describe("pet pointer interactions", () => {
  it("distinguishes click, drag, and native context-menu requests", () => {
    const activate = vi.fn();
    const drag = vi.fn();
    const context = vi.fn();
    render(
      <PetCharacter
        mood="idle"
        onActivate={activate}
        onRequestDrag={drag}
        onContextMenu={context}
      />,
    );
    const pet = screen.getByRole("button", {
      name: "点击 VocaPip 开始学习",
    });

    fireEvent.click(pet);
    expect(activate).toHaveBeenCalledTimes(1);

    fireEvent.pointerDown(pet, { button: 0, screenX: 10, screenY: 10 });
    fireEvent.pointerMove(pet, { buttons: 1, screenX: 18, screenY: 10 });
    fireEvent.click(pet);
    expect(drag).toHaveBeenCalledTimes(1);
    expect(activate).toHaveBeenCalledTimes(1);

    fireEvent.contextMenu(pet, { clientX: 12, clientY: 16 });
    expect(context).toHaveBeenCalledWith(12, 16);
  });
});
describe("library and settings", () => {
  it("filters the local demo library", async () => {
    const user = userEvent.setup();
    render(<LibraryView />);
    const search = await screen.findByPlaceholderText("搜索单词或中文释义");
    await user.type(search, "coherent");
    expect(await screen.findByText("coherent")).toBeTruthy();
    await waitFor(() => expect(screen.queryByText("steady")).toBeNull());
  });
  it("loads editable offline settings and applies topmost changes immediately", async () => {
    const user = userEvent.setup();
    render(<SettingsView />);
    expect(
      await screen.findByRole("heading", { name: "学习节奏" }),
    ).toBeTruthy();
    expect(screen.getByLabelText("提醒间隔")).toHaveProperty("value", "30");
    expect(
      screen
        .getByRole("switch", { name: "开机启动" })
        .getAttribute("aria-checked"),
    ).toBe("false");
    const topmost = screen.getByRole("switch", { name: "桌宠始终置顶" });
    expect(topmost.getAttribute("aria-checked")).toBe("false");
    await user.click(topmost);
    expect(await screen.findByText("已立即应用")).toBeTruthy();
    expect(topmost.getAttribute("aria-checked")).toBe("true");
    expect(screen.getByText("纯离线")).toBeTruthy();
  });
});
