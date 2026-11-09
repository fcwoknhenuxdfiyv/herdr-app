import { expect, test } from "bun:test";
import { readFile } from "node:fs/promises";

const source = await readFile(new URL("../defaults/frame/settings/main.js", import.meta.url), "utf8");

function setup(platform = "macos") {
  const controls = new Map<string, any>();
  function element() {
    const state: any = { nodes: [] };
    const builder: any = new Proxy(state, {
      get(target, key) {
        if (key in target) return target[key];
        return (...args: any[]) => {
          if (key === "id") controls.set(args[0], builder);
          if (key === "on_click") target.click = args[0];
          if (key === "child") target.nodes.push(args[0]);
          if (key === "children") target.nodes.push(...args[0]);
          return builder;
        };
      },
    });
    return builder;
  }
  let saved = JSON.stringify({ option_as_alt: "left", font_size: 13 });
  let task: Promise<void>;
  const values = {
    View: class {}, div: element, v_flex: element, h_flex: element,
    window: () => ({ platform }), appHome: () => "/test", settings: () => ({}),
    readTextFile: async () => saved,
    writeTextFile: async (_path: string, text: string, expected: string) => {
      expect(expected).toBe(saved);
      saved = text;
    },
  };
  const load = source.replace(/^import .*;\n/gm, "").replace("export default class Settings", "class Settings");
  const { Settings, sections } = new Function(...Object.keys(values), `${load}\nreturn { Settings, sections: SECTIONS };`)(...Object.values(values));
  const view = new Settings();
  view.person = {};
  const cx = { notify() {}, spawn(fn: any) { task = fn(cx); } };
  return {
    sections, view, controls,
    render() { view.keyboard({}); },
    renderTabWidth() { view.tabWidth({}); },
    async click(id: string) {
      controls.get(id).click({}, cx);
      await task;
      return JSON.parse(saved);
    },
  };
}

test("Option shortcut controls persist each side without changing terminal Option behavior", async () => {
  const state = setup();
  state.render();
  expect(state.controls.get("st-option-shortcuts-both").nodes[0].nodes).toContain("◉");
  for (const side of ["left", "right", "both"]) {
    expect(await state.click(`st-option-shortcuts-${side}`)).toEqual({
      option_as_alt: "left", option_shortcuts: side, font_size: 13,
    });
    state.render();
    expect(state.controls.get(`st-option-shortcuts-${side}`).nodes[0].nodes).toContain("◉");
  }
});

test("Minimum tab width updates, clamps and resets without changing other preferences", async () => {
  const state = setup();
  state.renderTabWidth();
  let person = await state.click("st-tab-min-width-plus");
  expect(person.tab_min_width).toBe(160);
  expect(person.option_as_alt).toBe("left");
  expect(person.font_size).toBe(13);
  for (let count = 0; count < 45; count++) person = await state.click("st-tab-min-width-plus");
  expect(person.tab_min_width).toBe(400);
  for (let count = 0; count < 45; count++) person = await state.click("st-tab-min-width-minus");
  expect(person.tab_min_width).toBe(0);
  person = await state.click("st-tab-min-width-reset");
  expect(person.tab_min_width).toBeUndefined();
  person = await state.click("st-tab-min-width-minus");
  expect(person.tab_min_width).toBe(140);
});

test("Keyboard settings are available only on macOS", () => {
  expect(setup("macos").sections.some((section: any) => section.id === "keyboard")).toBe(true);
  for (const platform of ["linux", "windows"]) {
    expect(setup(platform).sections.some((section: any) => section.id === "keyboard")).toBe(false);
  }
});
