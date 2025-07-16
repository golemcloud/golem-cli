/**
 * WAVE (WebAssembly Value Encoding) format utilities
 * Converts TypeScript values to WAVE format strings that golem-cli can parse
 */

/**
 * Converts a JavaScript value to WAVE format string
 * Based on golem-cli's WAVE parsing logic
 */
export function convertToWaveFormat(value: unknown, context?: { isEnum?: boolean }): string {
  if (value === null || value === undefined) {
    return "null";
  }

  const type = typeof value;

  switch (type) {
    case "string":
      // Enum values should be unquoted identifiers
      if (context?.isEnum) {
        return value as string;
      }
      // Regular strings need to be quoted
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
export function convertPayloadToWaveArgs(payload: { params: Array<{ value: unknown; typ?: unknown }> }): string[] {
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