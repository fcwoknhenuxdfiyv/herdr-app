import { expect, test } from "bun:test";
import { readFile } from "node:fs/promises";

const source = await readFile(new URL("../defaults/mods/tabs/main.js", import.meta.url), "utf8");

test("Tab minimum widths update live while retaining content sizing and horizontal scrolling", () => {
  const elements = new Map<string, any>();
  function element() {
    const state: any = { values: {}, nodes: [] };
    const builder: any = new Proxy(state, {
      get(target, key) {
        if (key in target) return target[key];
        return (...args: any[]) => {
          if (key === "id") elements.set(args[0], builder);
          if (key === "child") target.nodes.push(args[0]);
          if (key === "when") return args[0] ? args[1](builder) : builder;
          target.values[key] = args;
          return builder;
        };
      },
    });
    return builder;
  }
  let width: number | undefined;
  const values = {
    View: class {}, div: element, h_flex: element, svg: element, image: element,
    settings: () => ({ tab_min_width: width, zoom: 1 }), theme: () => ({}),
    spaces: () => [{ active: true, tabs: [{ id: "short", name: "1" }, { id: "long", name: "A longer tab name", active: true, zoomed: true, status: "working" }] }],
    panes: () => [], window: () => ({}), shared: () => [], layout: () => ({}),
    localStorage: { getItem: () => null },
  };
  const Tabs = new Function(...Object.keys(values), source.replace(/^import .*;\n/gm, "").replace("export default class Tabs", "return class Tabs"))(...Object.values(values));
  const view = new Tabs();
  view.init();
  for (const next of [undefined, 160, 0, 400]) {
    width = next;
    view.render();
    for (const id of ["tab-short", "tab-long"]) {
      expect(elements.get(id).values.min_w).toEqual([width ?? 150]);
      expect(elements.get(id).values.w).toBeUndefined();
      expect(elements.get(id).values.flex_none).toEqual([]);
    }
    const tab = elements.get("tab-long");
    const spacer = elements.get("tab-spacer-long");
    expect(spacer.values.flex_grow).toEqual([1]);
    expect(tab.nodes.indexOf(spacer)).toBe(1);
    expect(tab.nodes[2].values.spin).toEqual([900]);
    expect(tab.nodes[3]).toBe(elements.get("close-long"));
    expect(elements.get("tab-scroll").values.overflow_x_scrollbar).toEqual([]);
  }
});
