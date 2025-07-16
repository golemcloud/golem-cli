/**
 * WAVE (WebAssembly Value Encoding) format utilities
 * Converts TypeScript values to WAVE format strings that golem-cli can parse
 */

import { Parameter } from "@/types/component";

/**
 * Converts a JavaScript value to WAVE format string
 * Based on golem-cli's WAVE parsing logic
 */
export function convertToWaveFormat(
  value: unknown,
  context?: { isEnum?: boolean, isOption?: boolean },
): string {
  console.log("convertToWaveFormat called with:", { value, context });

  if (value === null || value === undefined) {
    if (context?.isOption) {
      return "none"
    }
    return "null";
  }

  const type = typeof value;

  switch (type) {
    case "string":
      // Enum values should be unquoted identifiers
      if (context?.isEnum) {
        console.log("Returning unquoted enum value:", value);
        return value as string;
      }
      // Regular strings need to be quoted
      console.log("Returning quoted string:", value);
      return `"${(value as string).replace(/"/g, '\\"')}"`;

    case "number":
      return String(value);

    case "boolean":
      return String(value);

    case "object":
      if (Array.isArray(value)) {
        // Arrays in WAVE format: [item1, item2, item3]
        const items = value.map(item => convertToWaveFormat(item)).join(", ");
        return `[${items}]`;
      } else {
        // Objects in WAVE format: {key1: value1, key2: value2}
        const entries = Object.entries(value as Record<string, unknown>)
          .map(([key, val]) => `${key}: ${convertToWaveFormat(val)}`)
          .join(", ");
        return `{${entries}}`;
      }

    default:
      // Fallback to JSON string representation
      return JSON.stringify(value);
  }
}

/**
 * Converts the current payload format to individual WAVE arguments
 * @param payload - The current payload format: { params: [{ value, typ }] }
 * @returns Array of WAVE-formatted argument strings
 */
export function convertPayloadToWaveArgs(payload: {
  params: Array<{ value: unknown; typ?: unknown }>;
}): string[] {
  return payload.params.map(param => convertToWaveFormat(param.value));
}

/**
 * Simplified converter that takes just the values (since CLI will handle type validation)
 * @param values - Array of raw values to convert to WAVE format
 * @returns Array of WAVE-formatted argument strings
 */
export function convertValuesToWaveArgs(values: unknown[]): string[] {
  return values.map(value => convertToWaveFormat(value));
}

/**
 * Smart converter that handles parameter types properly for WAVE format
 * @param value - The value to convert
 * @param parameter - The parameter definition with type information
 * @returns WAVE-formatted string
 */
export function convertToWaveFormatWithType(
  value: unknown,
  parameter?: Parameter,
): string {
  console.log("convertToWaveFormatWithType called with:", { value, parameter });

  if (value == null || value == undefined) {
    return "null";
  }

  // Handle different parameter types
  if (parameter?.typ?.type === "enum") {
    // Enum values are unquoted identifiers
    return convertToWaveFormat(value, { isEnum: true });
  }

  if (parameter?.typ?.type === "record") {
    // For records, check each field type
    if (typeof value === "object" && value !== null && !Array.isArray(value)) {
      const entries = Object.entries(value as Record<string, unknown>)
        .map(([key, val]) => {
          const field = parameter.typ.fields?.find((f: any) => f.name === key);
          if (field?.typ?.type === "enum") {
            return `${key}: ${convertToWaveFormat(val, { isEnum: true })}`;
          }
          if (field?.typ?.type === "option") {
            return `${key}: ${convertToWaveFormat(val, { isOption: true })}`;
          }
          return `${key}: ${convertToWaveFormat(val)}`;
        })
        .join(", ");
      return `{${entries}}`;
    }
  }

  if (parameter?.typ?.type === "option") {
    // Handle optional values
    if (value === null || value === undefined) {
      return "none";
    }
    return convertToWaveFormatWithType(value, {
      typ: parameter.typ.inner!,
      name: parameter.name,
      type: parameter.typ.type,
    });
  }

  // Default conversion
  console.log("Using default conversion for:", value, parameter?.typ?.type);
  return convertToWaveFormat(value);
}
