import { describe, expect, it } from "vitest";

import {
  EMPTY_STATE,
  contextFields,
  parseState,
  serializeState,
  splitValueUnit,
  swapUnits,
  toRequest,
  withContext,
  withField,
  type AppState,
  type ContextField,
} from "../../src/state";

// A stand-in for the catalogue's context list. The real one comes from Rust;
// these tests only need its shape.
const FIELDS: readonly ContextField[] = contextFields({
  context: [
    { key: "density", param: "d", description: "", input: { kind: "number", unit: "kg/m³" } },
    {
      key: "atmosphericPressure",
      param: "atm",
      description: "",
      input: { kind: "pressure", units: ["kPa", "bar"] },
    },
    {
      key: "referenceTemperature",
      param: "rt",
      description: "",
      input: { kind: "temperature", units: ["°C", "K"] },
    },
    {
      key: "compressibilityFactor",
      param: "z",
      description: "",
      input: { kind: "number", unit: null },
    },
  ],
});

const FULL: AppState = {
  value: "30",
  quantity: "pressure",
  from: "psig",
  to: "bar",
  sigFigs: "4",
  context: {
    density: { value: "998.2", unit: "" },
    atmosphericPressure: { value: "101.325", unit: "kPa" },
    referenceTemperature: { value: "-40", unit: "°C" },
    compressibilityFactor: { value: "0.95", unit: "" },
  },
};

describe("query string", () => {
  it("round-trips every field", () => {
    const search = serializeState(FULL, FIELDS);
    expect(parseState(search, FIELDS)).toEqual(FULL);
  });

  it("uses short, readable parameter names", () => {
    expect(serializeState(FULL, FIELDS)).toBe(
      "?v=30&q=pressure&from=psig&to=bar&sf=4&d=998.2&atm=101.325kPa&rt=-40%C2%B0C&z=0.95",
    );
  });

  it("serialises empty state as an empty string", () => {
    expect(serializeState(EMPTY_STATE, FIELDS)).toBe("");
    expect(parseState("", FIELDS)).toEqual(EMPTY_STATE);
  });

  it("ignores unknown, empty and malformed parameters", () => {
    const state = parseState("?v=1&nonsense=2&from=&atm=kPa&rt=%%%&d=&to=K", FIELDS);
    expect(state).toEqual({ ...EMPTY_STATE, value: "1", to: "K" });
  });

  it("keeps the first of duplicated parameters", () => {
    expect(parseState("?v=1&v=2", FIELDS).value).toBe("1");
  });

  it("keeps typed text that Rust will reject, so the error can be shown", () => {
    const state = parseState("?v=abc&d=heavy&sf=99", FIELDS);
    expect(state.value).toBe("abc");
    expect(state.sigFigs).toBe("99");
    expect(state.context["density"]).toEqual({ value: "heavy", unit: "" });
  });

  it("omits context fields that have a unit but no value", () => {
    const state = withContext(EMPTY_STATE, "atmosphericPressure", { value: "", unit: "kPa" });
    expect(serializeState(state, FIELDS)).toBe("");
  });
});

describe("splitValueUnit", () => {
  it.each([
    ["101.325kPa", { value: "101.325", unit: "kPa" }],
    ["1e5Pa", { value: "1e5", unit: "Pa" }],
    ["-40°C", { value: "-40", unit: "°C" }],
    [".5 bar", { value: ".5", unit: "bar" }],
    ["1000", { value: "1000", unit: "" }],
  ])("splits %s", (text, expected) => {
    expect(splitValueUnit(text)).toEqual(expected);
  });

  it("returns null without a leading number", () => {
    expect(splitValueUnit("kPa")).toBeNull();
    expect(splitValueUnit("")).toBeNull();
  });
});

describe("request", () => {
  it("sends plain numbers as text and quantities as value and unit", () => {
    expect(toRequest(FULL, FIELDS)).toEqual({
      value: "30",
      from: "psig",
      to: "bar",
      sigFigs: "4",
      context: {
        density: "998.2",
        atmosphericPressure: { value: "101.325", unit: "kPa" },
        referenceTemperature: { value: "-40", unit: "°C" },
        compressibilityFactor: "0.95",
      },
    });
  });

  it("omits significant figures when blank", () => {
    expect(toRequest({ ...FULL, sigFigs: " " }, FIELDS)).not.toHaveProperty("sigFigs");
  });
});

describe("transitions", () => {
  it("swaps units and keeps the value", () => {
    expect(swapUnits(FULL)).toEqual({ ...FULL, from: "bar", to: "psig" });
  });

  it("sets main fields without touching others", () => {
    expect(withField(FULL, "value", "12")).toEqual({ ...FULL, value: "12" });
  });

  it("removes a context field once it is cleared", () => {
    const cleared = withContext(FULL, "density", { value: "", unit: "" });
    expect(cleared.context).not.toHaveProperty("density");
    expect(FULL.context).toHaveProperty("density");
  });
});
