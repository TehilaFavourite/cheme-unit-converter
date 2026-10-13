// Wires state, the WebAssembly converter and the render functions together.
// Every change goes: event → new state → URL → convert in Rust → render.

import "./styles/tokens.css";
import "./styles/base.css";
import "./styles/components.css";

import {
  contextFields,
  parseState,
  serializeState,
  swapUnits,
  toRequest,
  withContext,
  withField,
  type AppState,
} from "./state";
import { buildContext, renderContext, type ContextDisplay } from "./ui/context";
import { bindForm, renderForm, type MainField } from "./ui/form";
import { buildReference } from "./ui/reference";
import { bindResult, renderLoadFailure, renderResult } from "./ui/result";
import { loadConverter, type Catalogue, type ConvertResponse, type Converter } from "./wasm";

const MAIN_FIELDS: ReadonlySet<string> = new Set<MainField>(["value", "from", "to", "sigFigs"]);

function isMainField(field: string): field is MainField {
  return MAIN_FIELDS.has(field);
}

/**
 * Fills in what a bare or partial URL leaves open: the quantity follows the
 * typed units, and a quantity without units starts from its first two units.
 * These are choices of unit, never values. Typed units are shown as Rust's
 * labels (Sm3/h becomes Sm³/h).
 */
function withDefaults(typed: AppState, converter: Converter): AppState {
  const { quantities } = converter.catalogue;
  // Show context units as Rust labels, so "rt=0C" selects "°C".
  const context = Object.fromEntries(
    Object.entries(typed.context).map(([key, entry]) => [
      key,
      { ...entry, unit: labelOf(converter, entry.unit) },
    ]),
  );
  const state = {
    ...typed,
    from: labelOf(converter, typed.from),
    to: labelOf(converter, typed.to),
    context,
  };
  const resolved = [state.from, state.to]
    .map((text) => converter.resolveUnit(text))
    .find((response) => response.ok);
  const known = quantities.some(({ key }) => key === state.quantity);
  const quantityKey = known ? state.quantity : (resolved?.quantity ?? quantities[0]?.key ?? "");
  const next = { ...state, quantity: quantityKey };
  if (state.from !== "" || state.to !== "") {
    return next;
  }
  const units = quantities.find(({ key }) => key === quantityKey)?.units ?? [];
  return { ...next, from: units[0]?.label ?? "", to: units[1]?.label ?? units[0]?.label ?? "" };
}

/** Returns Rust's display label for typed unit text, or the text unchanged. */
function labelOf(converter: Converter, text: string): string {
  if (text.trim() === "") {
    return text;
  }
  const resolved = converter.resolveUnit(text);
  return resolved.ok ? resolved.label : text;
}

/** Works out which context fields to show, with which messages. */
function contextDisplay(state: AppState, response: ConvertResponse): ContextDisplay {
  const filled = Object.keys(state.context);
  const visible = new Set(filled);
  const messages = new Map<string, string>();
  if (!response.ok) {
    const { field, missing, message } = response.error;
    const target = missing ?? (isMainField(field) ? null : field);
    if (target !== null) {
      visible.add(target);
      messages.set(target, message);
    }
  }
  const used = response.ok ? new Set(response.used) : null;
  const unused = new Set(
    used === null
      ? []
      : filled.filter((key) => state.context[key]?.value.trim() !== "" && !used.has(key)),
  );
  return { visible, messages, unused };
}

function mainMessages(response: ConvertResponse): Map<MainField, string> {
  const messages = new Map<MainField, string>();
  if (!response.ok && response.error.kind !== "emptyInput" && isMainField(response.error.field)) {
    messages.set(response.error.field, response.error.message);
  }
  return messages;
}

function unitsOf(catalogue: Catalogue, quantity: string): readonly string[] {
  return (
    catalogue.quantities.find(({ key }) => key === quantity)?.units.map(({ label }) => label) ?? []
  );
}

async function start(): Promise<void> {
  const converter = await loadConverter();
  const { catalogue } = converter;
  const fields = contextFields(catalogue);

  const form = bindForm(catalogue);
  const result = bindResult();
  buildReference(catalogue);

  let state = withDefaults(parseState(location.search, fields), converter);
  // True while the user is typing in one field, so a burst of keystrokes
  // becomes one history entry instead of one per character.
  let editing = false;

  const urlFor = (next: AppState): string =>
    `${location.pathname}${serializeState(next, fields)}${location.hash}`;

  const contextView = buildContext(catalogue.context, {
    onInput(key, entry, commit) {
      update(withContext(state, key, entry), commit ? "commit" : "edit");
    },
    onClear(key) {
      update(withContext(state, key, { value: "", unit: "" }), "commit");
    },
  });

  function render(): void {
    const response = converter.convert(toRequest(state, fields));
    renderForm(form, state, catalogue, mainMessages(response));
    renderContext(contextView, state, contextDisplay(state, response));
    renderResult(result, state, response);
  }

  /**
   * Applies a change. `edit` is typing: the first keystroke of a burst adds a
   * history entry and later ones replace it. `commit` is a discrete action
   * (choosing from a list, swapping) and always adds an entry.
   */
  function update(next: AppState, kind: "edit" | "commit"): void {
    state = next;
    const url = urlFor(state);
    const current = `${location.pathname}${location.search}${location.hash}`;
    if (url !== current) {
      if (kind === "commit" || !editing) {
        history.pushState(null, "", url);
      } else {
        history.replaceState(null, "", url);
      }
    }
    editing = kind === "edit";
    render();
  }

  const endEdit = (): void => {
    editing = false;
  };

  form.form.addEventListener("submit", (event) => {
    event.preventDefault();
  });
  form.value.addEventListener("input", () => {
    update(withField(state, "value", form.value.value), "edit");
  });
  form.sigFigs.addEventListener("input", () => {
    update(withField(state, "sigFigs", form.sigFigs.value), "edit");
  });
  for (const [input, field] of [
    [form.from, "from"],
    [form.to, "to"],
  ] as const) {
    input.addEventListener("input", () => {
      let next = withField(state, field, input.value);
      // A unit typed from another quantity switches the quantity to match.
      const resolved = converter.resolveUnit(input.value);
      if (resolved.ok && resolved.quantity !== state.quantity) {
        next = withField(next, "quantity", resolved.quantity);
      }
      update(next, "edit");
    });
  }
  for (const input of [form.value, form.sigFigs]) {
    input.addEventListener("change", endEdit);
  }
  // On leaving a unit field, show the unit as Rust labels it (Sm3/h → Sm³/h).
  for (const [input, field] of [
    [form.from, "from"],
    [form.to, "to"],
  ] as const) {
    input.addEventListener("change", () => {
      endEdit();
      const label = labelOf(converter, input.value);
      if (label !== state[field]) {
        update(withField(state, field, label), "edit");
        endEdit();
      }
    });
  }
  form.quantity.addEventListener("change", () => {
    const [from = "", to = from] = unitsOf(catalogue, form.quantity.value);
    update({ ...state, quantity: form.quantity.value, from, to }, "commit");
  });
  form.swap.addEventListener("click", () => {
    update(swapUnits(state), "commit");
  });
  window.addEventListener("popstate", () => {
    editing = false;
    state = withDefaults(parseState(location.search, fields), converter);
    render();
  });

  history.replaceState(null, "", urlFor(state));
  render();
}

start().catch((error: unknown) => {
  renderLoadFailure(bindResult());
  console.error(error);
});
