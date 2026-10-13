// Context fields (density, atmospheric pressure, ...). One field per catalogue
// entry is built up front and kept hidden until it holds a value or Rust
// reports it missing. Nothing is ever pre-filled.

import type { AppState, ContextValue } from "../state";
import type { ContextInfo } from "../wasm";
import { byId, create, sentenceCase, setMessage, setValue } from "./dom";

interface FieldView {
  readonly info: ContextInfo;
  readonly container: HTMLElement;
  readonly input: HTMLInputElement;
  readonly unit: HTMLSelectElement | null;
  readonly note: HTMLElement;
  readonly message: HTMLElement;
}

export interface ContextView {
  readonly fieldset: HTMLFieldSetElement;
  readonly fields: readonly FieldView[];
}

/** What to show for the context fields after a conversion. */
export interface ContextDisplay {
  /** Fields to show: those with a value, plus the one Rust asked for. */
  readonly visible: ReadonlySet<string>;
  /** Messages from Rust, keyed by the field they concern. */
  readonly messages: ReadonlyMap<string, string>;
  /** Filled fields the current conversion does not use. */
  readonly unused: ReadonlySet<string>;
}

export interface ContextHandlers {
  onInput(key: string, entry: ContextValue, commit: boolean): void;
  onClear(key: string): void;
}

function buildField(info: ContextInfo, handlers: ContextHandlers): FieldView {
  const id = `context-${info.param}`;
  const input = create("input", {
    id,
    class: "input input-number",
    type: "text",
    inputmode: "decimal",
    spellcheck: "false",
    "aria-describedby": `${id}-message ${id}-note`,
  });
  const unit =
    info.input.kind === "number"
      ? null
      : create(
          "select",
          {
            id: `${id}-unit`,
            class: "input input-unit",
            "aria-label": `Unit of ${info.description}`,
          },
          [
            create("option", { value: "" }, ["Unit"]),
            ...info.input.units.map((label) => create("option", { value: label }, [label])),
          ],
        );
  const clear = create("button", { class: "button button-quiet", type: "button" }, ["Clear"]);
  clear.setAttribute("aria-label", `Clear ${info.description}`);
  const note = create("p", { id: `${id}-note`, class: "note", hidden: "" }, [
    "Not used by this conversion. ",
    clear,
  ]);
  const message = create("p", { id: `${id}-message`, class: "message", hidden: "" });
  const control = create("div", { class: "control" }, unit === null ? [input] : [input, unit]);
  const container = create("div", { class: "field", hidden: "", "data-context": info.key }, [
    create("label", { for: id }, [sentenceCase(info.description)]),
    control,
    message,
    note,
  ]);

  const entry = (): ContextValue => ({ value: input.value, unit: unit?.value ?? "" });
  input.addEventListener("input", () => {
    handlers.onInput(info.key, entry(), false);
  });
  input.addEventListener("change", () => {
    handlers.onInput(info.key, entry(), true);
  });
  unit?.addEventListener("change", () => {
    handlers.onInput(info.key, entry(), true);
  });
  clear.addEventListener("click", () => {
    handlers.onClear(info.key);
  });

  return { info, container, input, unit, note, message };
}

export function buildContext(
  context: readonly ContextInfo[],
  handlers: ContextHandlers,
): ContextView {
  const fieldset = byId("context", HTMLFieldSetElement);
  const fields = context.map((info) => buildField(info, handlers));
  byId("context-fields", HTMLElement).replaceChildren(...fields.map(({ container }) => container));
  return { fieldset, fields };
}

export function renderContext(view: ContextView, state: AppState, display: ContextDisplay): void {
  for (const field of view.fields) {
    const { key } = field.info;
    const entry = state.context[key];
    setValue(field.input, entry?.value ?? "");
    if (field.unit !== null) {
      setValue(field.unit, entry?.unit ?? "");
    }
    field.container.hidden = !display.visible.has(key);
    field.note.hidden = !display.unused.has(key);
    field.container.classList.toggle("is-unused", display.unused.has(key));
    setMessage(field.input, field.message, display.messages.get(key) ?? null);
  }
  view.fieldset.hidden = display.visible.size === 0;
}
