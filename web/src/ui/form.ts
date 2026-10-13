// The main inputs: value, quantity, from/to units, swap and significant figures.

import type { AppState } from "../state";
import type { Catalogue } from "../wasm";
import { byId, create, sentenceCase, setMessage, setValue } from "./dom";

/** The main fields that can carry an inline message. */
export type MainField = "value" | "from" | "to" | "sigFigs";

export interface FormView {
  readonly form: HTMLFormElement;
  readonly value: HTMLInputElement;
  readonly quantity: HTMLSelectElement;
  readonly from: HTMLInputElement;
  readonly to: HTMLInputElement;
  readonly sigFigs: HTMLInputElement;
  readonly swap: HTMLButtonElement;
  readonly unitOptions: HTMLDataListElement;
  readonly messages: Readonly<Record<MainField, HTMLElement>>;
}

export function bindForm(catalogue: Catalogue): FormView {
  const view: FormView = {
    form: byId("converter", HTMLFormElement),
    value: byId("value", HTMLInputElement),
    quantity: byId("quantity", HTMLSelectElement),
    from: byId("from", HTMLInputElement),
    to: byId("to", HTMLInputElement),
    sigFigs: byId("sig-figs", HTMLInputElement),
    swap: byId("swap", HTMLButtonElement),
    unitOptions: byId("unit-options", HTMLDataListElement),
    messages: {
      value: byId("value-message", HTMLElement),
      from: byId("from-message", HTMLElement),
      to: byId("to-message", HTMLElement),
      sigFigs: byId("sig-figs-message", HTMLElement),
    },
  };
  view.quantity.replaceChildren(
    ...catalogue.quantities.map(({ key, name }) =>
      create("option", { value: key }, [sentenceCase(name)]),
    ),
  );
  view.sigFigs.placeholder = `up to ${String(catalogue.defaultSignificantFigures)}`;
  return view;
}

export function renderForm(
  view: FormView,
  state: AppState,
  catalogue: Catalogue,
  messages: ReadonlyMap<MainField, string>,
): void {
  setValue(view.value, state.value);
  setValue(view.quantity, state.quantity);
  setValue(view.from, state.from);
  setValue(view.to, state.to);
  setValue(view.sigFigs, state.sigFigs);

  // Suggest the units of the selected quantity. Any accepted spelling may
  // still be typed; Rust resolves it.
  if (view.unitOptions.dataset["quantity"] !== state.quantity) {
    const quantity = catalogue.quantities.find(({ key }) => key === state.quantity);
    view.unitOptions.replaceChildren(
      ...(quantity?.units ?? []).map(({ label }) => create("option", { value: label })),
    );
    view.unitOptions.dataset["quantity"] = state.quantity;
  }

  setMessage(view.value, view.messages.value, messages.get("value") ?? null);
  setMessage(view.from, view.messages.from, messages.get("from") ?? null);
  setMessage(view.to, view.messages.to, messages.get("to") ?? null);
  setMessage(view.sigFigs, view.messages.sigFigs, messages.get("sigFigs") ?? null);
}
