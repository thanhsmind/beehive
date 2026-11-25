# Herded read-only worker guard refuses an rg search whose pattern names a bee verb

## Symptom

A read-only gather worker ran rg with a search pattern containing the words of a bee verb (team show); the worker-guard denied it as running that verb

## Evidence

job-1795621685412-3113025-1 report-1.md: PreToolUse worker-guard denied: team show is not allowed — this herded run is read-only

## Output

```text
`team show` is not allowed — this herded run is read-only.
```

## Suspected root cause

The worker guard matches bee verb phrases anywhere in the Bash command text, including inside rg search pattern arguments, not only as an invoked bee command

## Command

```sh
bee herding run --task-file - --json --session-model haiku --read-only
```

## Exit code

not given

## Environment

- bee version: 2.122.0
- OS: linux x86_64
