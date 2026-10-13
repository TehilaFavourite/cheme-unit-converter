// The collapsible list of every unit and its accepted spellings.

import type { Catalogue } from "../wasm";
import { byId, create, sentenceCase } from "./dom";

export function buildReference(catalogue: Catalogue): void {
  const sections = catalogue.quantities.map(({ key, name, units }) => {
    const headingId = `reference-${key}`;
    const rows = units.map(({ label, aliases }) =>
      create("tr", {}, [
        create("th", { scope: "row", class: "mono" }, [label]),
        create("td", { class: "mono" }, [aliases.length === 0 ? "—" : aliases.join(", ")]),
      ]),
    );
    return create("section", { "aria-labelledby": headingId }, [
      create("h3", { id: headingId }, [sentenceCase(name)]),
      create("table", {}, [
        create("thead", {}, [
          create("tr", {}, [
            create("th", { scope: "col" }, ["Unit"]),
            create("th", { scope: "col" }, ["Also accepted"]),
          ]),
        ]),
        create("tbody", {}, rows),
      ]),
    ]);
  });
  byId("reference", HTMLElement).replaceChildren(...sections);
}
