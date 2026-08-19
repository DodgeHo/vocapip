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
import { api } from "./lib/api";
import { PetCharacter, resolveCustomPetImage } from "./components/PetCharacter";
import { LibraryView } from "./views/LibraryView";
import { PetView } from "./views/PetView";
import { SettingsView } from "./views/SettingsView";
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});
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

  it("keeps system appearance classes and custom image fallback stable", () => {
    const { container } = render(
      <>
        <PetCharacter mood="idle" appearanceId="coral" />
        <PetCharacter
          mood="correct"
          appearanceId="custom"
          customAssets={{ default: "data:image/png;base64,custom" }}
        />
      </>,
    );

    expect(container.querySelectorAll(".pet-appearance-coral")).toHaveLength(1);
    expect(
      container.querySelector(".pet-custom-image")?.getAttribute("src"),
    ).toBe("data:image/png;base64,custom");
    expect(
      resolveCustomPetImage(
        { default: "default", idle: "idle", correct: "correct" },
        "correct",
      ),
    ).toBe("correct");
    expect(resolveCustomPetImage(undefined, "idle")).toBeNull();
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

  it("shows the local empty-example state in the library", async () => {
    const card = {
      id: "vocabularybook:middle-school:ab12",
      word: "abandon",
      phonetic: undefined,
      partOfSpeech: "v.",
      definitionZh: "放弃",
      exampleEn: "",
      exampleZh: "",
      tier: "T1" as const,
      frequencyRank: 0,
      status: "new" as const,
      favorite: false,
    };
    vi.spyOn(api, "listLibrary").mockResolvedValue({
      items: [card],
      counts: {
        all: 1,
        new: 1,
        learning: 0,
        review: 0,
        relearning: 0,
        favorite: 0,
        recent: 0,
      },
    });

    render(<LibraryView />);
    expect((await screen.findAllByText("暂无本地例句")).length).toBe(2);
  });

  it("keeps speech and appearance settings in the editable settings model", async () => {
    const user = userEvent.setup();
    render(<SettingsView />);
    await screen.findByRole("heading", { name: "学习节奏" });

    const speech = screen.getByRole("switch", { name: "自动朗读单词" });
    expect(speech.getAttribute("aria-checked")).toBe("false");
    await user.click(speech);
    expect(speech.getAttribute("aria-checked")).toBe("true");

    const coral = screen.getByRole("radio", { name: "Coral" });
    await user.click(coral);
    expect(coral.getAttribute("aria-checked")).toBe("true");
    await user.click(screen.getByRole("button", { name: "已立即应用" }));
    await screen.findByText("已保存");

    const saved = await api.bootstrap();
    expect(saved.settings.speechEnabled).toBe(true);
    expect(saved.settings.petAppearanceId).toBe("coral");
  });

  it("uses the speech command when the word speaker is pressed", async () => {
    const user = userEvent.setup();
    const speak = vi.spyOn(api, "speakText").mockResolvedValue();
    await api.completePlacement({ selfReportedTier: "T2" });
    render(<PetView />);

    await user.click(
      await screen.findByRole("button", { name: "点击 VocaPip 开始学习" }),
    );
    await user.click(screen.getByRole("button", { name: "朗读单词" }));
    expect(speak).toHaveBeenCalledWith(expect.stringMatching(/steady|notice/));
  });

  it("automatically reads a card and stops when the bubble closes", async () => {
    const user = userEvent.setup();
    const current = await api.bootstrap();
    await api.updateSettings({
      ...current.settings,
      speechEnabled: true,
    });
    await api.completePlacement({ selfReportedTier: "T2" });
    const speak = vi.spyOn(api, "speakText").mockResolvedValue();
    const stop = vi.spyOn(api, "stopSpeaking").mockResolvedValue();

    render(<PetView />);
    await user.click(
      await screen.findByRole("button", { name: "点击 VocaPip 开始学习" }),
    );
    await waitFor(() =>
      expect(speak).toHaveBeenCalledWith(
        expect.stringMatching(/steady|notice/),
      ),
    );

    await user.click(screen.getByRole("button", { name: "关闭" }));
    expect(stop).toHaveBeenCalled();
  });

  it("uploads one custom image, selects it, and restores Mint after removal", async () => {
    const user = userEvent.setup();
    class MockImage {
      naturalWidth = 256;
      naturalHeight = 256;
      onload?: () => void;
      onerror?: () => void;

      set src(_value: string) {
        queueMicrotask(() => this.onload?.());
      }
    }
    vi.stubGlobal("Image", MockImage);
    render(<SettingsView />);
    await screen.findByRole("heading", { name: "学习节奏" });

    const file = new File(["pet"], "pet.png", { type: "image/png" });
    await user.upload(screen.getByLabelText("上传通用图片"), file);

    expect(await screen.findByText("图片已保存在本机并启用")).toBeTruthy();
    expect(
      screen
        .getByRole("radio", { name: "我的图片" })
        .getAttribute("aria-checked"),
    ).toBe("true");

    await user.click(screen.getByRole("button", { name: "删除图片：通用" }));
    expect(await screen.findByText("图片已删除")).toBeTruthy();
    expect(
      screen.getByRole("radio", { name: "Mint" }).getAttribute("aria-checked"),
    ).toBe("true");
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
