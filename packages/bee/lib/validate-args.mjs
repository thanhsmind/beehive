// validate-args.mjs — the one arg validator both the future unified
// dispatcher (harness-integration-2, bee.mjs) and the extended
// bee-write-guard.mjs (harness-integration-3) import. Given a command-registry
// entry and a parsed-args object, decide whether the call is well-formed
// against the entry's JSON-Schema `parameters` — before anything dispatches
// or executes. Never throws: always returns a structured result.

/**
 * Structural check that a `parameters` value is JSON-Schema in the exact
 * shape D3 requires: {type:"object", properties, required}, every `required`
 * name present in `properties`, every property carrying a `type`.
 */
export function isValidParameterSchema(schema) {
  if (!schema || typeof schema !== 'object' || Array.isArray(schema)) return false;
  if (schema.type !== 'object') return false;
  if (!schema.properties || typeof schema.properties !== 'object' || Array.isArray(schema.properties)) {
    return false;
  }
  if (!Array.isArray(schema.required)) return false;
  for (const field of schema.required) {
    if (typeof field !== 'string' || !Object.prototype.hasOwnProperty.call(schema.properties, field)) {
      return false;
    }
  }
  for (const propSchema of Object.values(schema.properties)) {
    if (!propSchema || typeof propSchema.type !== 'string') return false;
  }
  return true;
}

function isPresent(value) {
  return value !== undefined && value !== null && value !== '';
}

// CLI flags arrive as strings (argv parsing never produces real booleans or
// numbers) - a schema of type "boolean"/"number" must still accept the CLI's
// own string encoding of them, not just a native JS boolean/number.
function typeMatches(jsonType, value) {
  switch (jsonType) {
    case 'string':
      return typeof value === 'string';
    case 'boolean':
      return typeof value === 'boolean' || value === 'true' || value === 'false';
    case 'number':
    case 'integer':
      if (typeof value === 'number') return Number.isFinite(value);
      return typeof value === 'string' && value.trim() !== '' && Number.isFinite(Number(value));
    case 'array':
      return Array.isArray(value) || typeof value === 'string'; // comma-separated CLI convention
    default:
      return true;
  }
}

/**
 * Validate parsedArgs against a command registry entry's parameters schema.
 * Collects EVERY problem (every missing required field, every wrong-typed or
 * out-of-enum value) instead of returning at the first one — ce-1 (cli-
 * ergonomics D1): one refusal should name everything wrong with a call, not
 * make the caller fix-and-retry one flag at a time.
 * @param {object} commandEntry - a COMMAND_REGISTRY entry (needs .name, .parameters)
 * @param {object} parsedArgs - flag name -> value (as parsed from argv)
 * @returns {{ok:true}|{ok:false, error:{field:string|null, reason:string, command:string|null}, problems:{field:string|null, reason:string}[]}}
 *   `error` is always the FIRST problem (test_bee_cli.mjs:325 pins its exact
 *   shape) — `problems` is the additive full list, same order `error` comes
 *   from (required misses first, in schema.required order, then value
 *   problems in parsedArgs iteration order).
 */
export function validate(commandEntry, parsedArgs = {}) {
  const command = commandEntry && typeof commandEntry.name === 'string' ? commandEntry.name : null;
  const schema = commandEntry && commandEntry.parameters;
  const args = parsedArgs && typeof parsedArgs === 'object' ? parsedArgs : {};

  if (!isValidParameterSchema(schema)) {
    const problem = { field: null, reason: 'command has no valid JSON-Schema parameters' };
    return { ok: false, error: { ...problem, command }, problems: [problem] };
  }

  const problems = [];

  for (const field of schema.required) {
    if (!isPresent(args[field])) {
      problems.push({ field, reason: 'required, missing' });
    }
  }

  for (const [field, value] of Object.entries(args)) {
    if (value === undefined) continue;
    const propSchema = schema.properties[field];
    if (!propSchema) continue; // unknown-flag rejection is the dispatcher/hook's own concern
    if (!typeMatches(propSchema.type, value)) {
      problems.push({ field, reason: `invalid type, expected ${propSchema.type}` });
      continue; // a mistyped value can't also be enum-checked meaningfully
    }
    // Enum enforcement is scoped to REQUIRED fields only (DB3 guard,
    // command-registry.mjs:1024-1031/641-648): every state.*/backlog.add
    // entry declares `required: []` and `type: 'string'` everywhere,
    // specifically to keep the generic validator from ever preempting the
    // handler's own STDERR-routed checks — but several of those same
    // entries still carry an `enum` annotation for documentation purposes
    // (e.g. backlog.add's severity). Enforcing enum on ANY present field
    // would silently re-open the STDOUT leak DB3 closed. Gating on
    // schema.required keeps this additive: today only fields already
    // required (cells.tier's tier, etc.) get enum-checked here.
    if (schema.required.includes(field) && Array.isArray(propSchema.enum) && !propSchema.enum.includes(value)) {
      problems.push({ field, reason: `invalid value, expected one of ${propSchema.enum.join(', ')}` });
    }
  }

  if (problems.length === 0) return { ok: true };

  const first = problems[0];
  return {
    ok: false,
    error: { field: first.field, reason: first.reason, command },
    problems,
  };
}
