// The only module that touches the generated WebAssembly bindings. It gives
// the Rust API (docs/adr/0003-wasm-boundary-design.md) TypeScript types and
// turns anything unexpected into an `internal` error instead of throwing.

import init, {
  catalogue as rawCatalogue,
  convert as rawConvert,
  resolveUnit as rawResolveUnit,
} from "./generated/converter.js";

export interface UnitInfo {
  readonly label: string;
  readonly aliases: readonly string[];
}

export interface QuantityInfo {
  readonly key: string;
  readonly name: string;
  readonly units: readonly UnitInfo[];
}

export type ContextInputInfo =
  | { readonly kind: "number"; readonly unit: string | null }
  | { readonly kind: "pressure" | "temperature"; readonly units: readonly string[] };

export interface ContextInfo {
  readonly key: string;
  readonly param: string;
  readonly description: string;
  readonly input: ContextInputInfo;
}

export interface Catalogue {
  readonly quantities: readonly QuantityInfo[];
  readonly context: readonly ContextInfo[];
  readonly maxSignificantFigures: number;
  readonly defaultSignificantFigures: number;
}

export type ErrorKind =
  | "emptyInput"
  | "invalidNumber"
  | "unrecognizedUnit"
  | "ambiguousPrefix"
  | "invalidSigFigs"
  | "wrongContextUnit"
  | "incompatibleUnits"
  | "missingContext"
  | "invalidContext"
  | "outOfRange"
  | "negativeValue"
  | "negativePressure"
  | "belowAbsoluteZero"
  | "nonFiniteValue"
  | "resultOverflow"
  | "internal";

/** `value`, `from`, `to`, `sigFigs`, or a context key. */
export type FieldKey = string;

export interface ApiError {
  readonly kind: ErrorKind;
  readonly message: string;
  readonly field: FieldKey;
  readonly missing?: string;
}

export interface ConvertSuccess {
  readonly ok: true;
  readonly value: number;
  readonly formatted: string;
  readonly from: string;
  readonly to: string;
  readonly quantity: string;
  readonly used: readonly string[];
}

export interface ConvertFailure {
  readonly ok: false;
  readonly error: ApiError;
}

export type ConvertResponse = ConvertSuccess | ConvertFailure;

/** A context value: plain numbers are text; quantities are text plus a unit. */
export type ContextRequestValue = string | { readonly value: string; readonly unit: string };

export interface ConvertRequest {
  readonly value: string;
  readonly from: string;
  readonly to: string;
  readonly sigFigs?: string;
  readonly context: Readonly<Record<string, ContextRequestValue>>;
}

export type ResolveResponse =
  | { readonly ok: true; readonly label: string; readonly quantity: string }
  | { readonly ok: false; readonly error: ApiError };

export interface Converter {
  readonly catalogue: Catalogue;
  convert(request: ConvertRequest): ConvertResponse;
  resolveUnit(text: string): ResolveResponse;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function internalError(message: string): ConvertFailure {
  return { ok: false, error: { kind: "internal", message, field: "value" } };
}

/**
 * Loads the WebAssembly module and returns the typed API. Rust guarantees the
 * shapes (see the wasm-bindgen tests); the checks here only guard against a
 * stale or mismatched build.
 */
export async function loadConverter(): Promise<Converter> {
  await init();
  const catalogue: unknown = rawCatalogue();
  if (!isRecord(catalogue) || !Array.isArray(catalogue["quantities"])) {
    throw new Error("The converter module returned an unexpected catalogue.");
  }
  return {
    catalogue: catalogue as unknown as Catalogue,
    convert(request) {
      const response: unknown = rawConvert(request);
      if (!isRecord(response) || typeof response["ok"] !== "boolean") {
        return internalError("The converter returned an unexpected response.");
      }
      return response as unknown as ConvertResponse;
    },
    resolveUnit(text) {
      const response: unknown = rawResolveUnit(text);
      if (!isRecord(response) || typeof response["ok"] !== "boolean") {
        return internalError("The converter returned an unexpected response.");
      }
      return response as unknown as ResolveResponse;
    },
  };
}
