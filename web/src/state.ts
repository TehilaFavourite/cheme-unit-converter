// Application state and its query-string form. Pure functions only: no DOM
// and no WebAssembly, so this module can be tested in Node.
//
// The state holds exactly what the user typed. Rust parses and validates it;
// nothing here interprets numbers or units.

import type { Catalogue, ContextRequestValue, ConvertRequest } from "./wasm";

/** A context value as typed. `unit` is empty for plain-number fields. */
export interface ContextValue {
  readonly value: string;
  readonly unit: string;
}

export interface AppState {
  readonly value: string;
  /** Quantity key from the catalogue, or empty. */
  readonly quantity: string;
  readonly from: string;
  readonly to: string;
  readonly sigFigs: string;
  /** Context fields the user has filled in, keyed by context key. */
  readonly context: Readonly<Record<string, ContextValue>>;
}

/** How each context field appears in the query string. */
export interface ContextField {
  readonly key: string;
  readonly param: string;
  /** Whether the field holds a value and a unit, rather than a plain number. */
  readonly hasUnit: boolean;
}

export function contextFields(catalogue: Pick<Catalogue, "context">): readonly ContextField[] {
  return catalogue.context.map(({ key, param, input }) => ({
    key,
    param,
    hasUnit: input.kind !== "number",
  }));
}

type MainField = "value" | "quantity" | "from" | "to" | "sigFigs";

const MAIN_PARAMS: Readonly<Record<MainField, string>> = {
  value: "v",
  quantity: "q",
  from: "from",
  to: "to",
  sigFigs: "sf",
};

const MAIN_FIELDS = Object.keys(MAIN_PARAMS) as readonly MainField[];

// A leading decimal number, as in "101.325kPa" or "1e5Pa". Only used to split
// the query-string form of a value and unit; Rust parses both parts.
const NUMBER_PREFIX = /^([+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)(.*)$/;

/** Splits "101.325kPa" into value and unit. Returns null if there is no number. */
export function splitValueUnit(text: string): ContextValue | null {
  const match = NUMBER_PREFIX.exec(text.trim());
  if (match === null) {
    return null;
  }
  const [, value = "", unit = ""] = match;
  return { value, unit: unit.trim() };
}

/** Reads state from a query string. Malformed parameters are ignored. */
export function parseState(search: string, fields: readonly ContextField[]): AppState {
  const params = new URLSearchParams(search);
  const read = (name: string): string => params.get(name)?.trim() ?? "";

  const context: Record<string, ContextValue> = {};
  for (const { key, param, hasUnit } of fields) {
    const text = read(param);
    if (text === "") {
      continue;
    }
    const parsed = hasUnit ? splitValueUnit(text) : { value: text, unit: "" };
    if (parsed !== null) {
      context[key] = parsed;
    }
  }

  return {
    value: read(MAIN_PARAMS.value),
    quantity: read(MAIN_PARAMS.quantity),
    from: read(MAIN_PARAMS.from),
    to: read(MAIN_PARAMS.to),
    sigFigs: read(MAIN_PARAMS.sigFigs),
    context,
  };
}

/** Writes state as a query string ("" when empty), in a stable order. */
export function serializeState(state: AppState, fields: readonly ContextField[]): string {
  const params = new URLSearchParams();
  for (const field of MAIN_FIELDS) {
    const text = state[field].trim();
    if (text !== "") {
      params.set(MAIN_PARAMS[field], text);
    }
  }
  for (const { key, param } of fields) {
    const entry = state.context[key];
    if (entry !== undefined && entry.value.trim() !== "") {
      params.set(param, `${entry.value.trim()}${entry.unit.trim()}`);
    }
  }
  const query = params.toString();
  return query === "" ? "" : `?${query}`;
}

/** Builds the request Rust expects from the typed state. */
export function toRequest(state: AppState, fields: readonly ContextField[]): ConvertRequest {
  const context: Record<string, ContextRequestValue> = {};
  for (const { key, hasUnit } of fields) {
    const entry = state.context[key];
    if (entry !== undefined) {
      context[key] = hasUnit ? { value: entry.value, unit: entry.unit } : entry.value;
    }
  }
  const request = { value: state.value, from: state.from, to: state.to, context };
  return state.sigFigs.trim() === "" ? request : { ...request, sigFigs: state.sigFigs };
}

export function withField(state: AppState, field: MainField, text: string): AppState {
  return { ...state, [field]: text };
}

/** Sets a context field; a field with neither value nor unit is removed. */
export function withContext(state: AppState, key: string, entry: ContextValue): AppState {
  const others = Object.entries(state.context).filter(([other]) => other !== key);
  const cleared = entry.value.trim() === "" && entry.unit.trim() === "";
  return {
    ...state,
    context: Object.fromEntries(cleared ? others : [...others, [key, entry]]),
  };
}

export function swapUnits(state: AppState): AppState {
  return { ...state, from: state.to, to: state.from };
}
