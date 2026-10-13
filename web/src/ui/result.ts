// The result line and its copy button.

import type { AppState } from "../state";
import type { ConvertResponse } from "../wasm";
import { byId, create, sentenceCase } from "./dom";

export interface ResultView {
  readonly body: HTMLElement;
  readonly copy: HTMLButtonElement;
  readonly status: HTMLElement;
}

const STATUS_CLEAR_DELAY_MS = 2000;

export function bindResult(): ResultView {
  const view: ResultView = {
    body: byId("result", HTMLElement),
    copy: byId("copy", HTMLButtonElement),
    status: byId("copy-status", HTMLElement),
  };
  let timer: ReturnType<typeof setTimeout> | undefined;
  view.copy.addEventListener("click", () => {
    const text = view.copy.dataset["value"] ?? "";
    const report = (message: string): void => {
      view.status.textContent = message;
      clearTimeout(timer);
      timer = setTimeout(() => {
        view.status.textContent = "";
      }, STATUS_CLEAR_DELAY_MS);
    };
    navigator.clipboard.writeText(text).then(
      () => {
        report("Copied");
      },
      () => {
        report("Copying is not available in this browser");
      },
    );
  });
  return view;
}

export function renderResult(view: ResultView, state: AppState, response: ConvertResponse): void {
  if (response.ok) {
    view.body.replaceChildren(
      create("p", { class: "result-input" }, [`${state.value.trim()} ${response.from} =`]),
      create("p", { class: "result-output" }, [
        create("span", { class: "result-number" }, [response.formatted]),
        " ",
        create("span", { class: "result-unit" }, [response.to]),
      ]),
    );
    view.copy.disabled = false;
    view.copy.dataset["value"] = response.formatted;
    return;
  }
  // Errors are shown next to the responsible field; here only say why there
  // is no result. A blank required input is a prompt, not an error.
  const text =
    response.error.kind === "emptyInput" ? sentenceCase(response.error.message) : "No result";
  view.body.replaceChildren(create("p", { class: "result-empty" }, [text]));
  view.copy.disabled = true;
  view.copy.removeAttribute("data-value");
}

export function renderLoadFailure(view: ResultView): void {
  view.body.replaceChildren(
    create("p", { class: "result-empty" }, [
      "The converter could not be loaded. Reload the page, or use a browser with WebAssembly support.",
    ]),
  );
}
