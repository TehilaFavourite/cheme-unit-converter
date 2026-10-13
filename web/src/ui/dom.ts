// Small DOM helpers shared by the render functions.

/** Returns the element with `id`, checking that it has the expected type. */
export function byId<T extends HTMLElement>(id: string, type: abstract new () => T): T {
  const element = document.getElementById(id);
  if (!(element instanceof type)) {
    throw new Error(`#${id} is missing or is not a ${type.name}.`);
  }
  return element;
}

/** Creates an element with attributes and children. */
export function create<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  attributes: Readonly<Record<string, string>> = {},
  children: readonly (Node | string)[] = [],
): HTMLElementTagNameMap[K] {
  const element = document.createElement(tag);
  for (const [name, value] of Object.entries(attributes)) {
    element.setAttribute(name, value);
  }
  element.append(...children);
  return element;
}

/** Upper-cases the first letter of a message or name from Rust. */
export function sentenceCase(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1);
}

/** Sets an input's value only when it differs, so the caret is not moved. */
export function setValue(input: HTMLInputElement | HTMLSelectElement, value: string): void {
  if (input.value !== value) {
    input.value = value;
  }
}

/** Shows or clears the message under a control and marks the control invalid. */
export function setMessage(
  control: HTMLInputElement | HTMLSelectElement,
  message: HTMLElement,
  text: string | null,
): void {
  message.hidden = text === null;
  message.textContent = text === null ? "" : sentenceCase(text);
  if (text === null) {
    control.removeAttribute("aria-invalid");
  } else {
    control.setAttribute("aria-invalid", "true");
  }
}
