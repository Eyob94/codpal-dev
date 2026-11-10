---
name: write-tests
description: Write well-thought-out tests from a requirement or spec, before the code exists (test-first). Use when an issue or task asks you to write tests, add test coverage, or turn a spec into tests.
---

# Write tests from a requirement

You write tests **before** the code. The tests are the contract.
Another agent will write code to make them pass. It cannot change your tests.
So your tests must be correct, complete, and clear.

Follow the steps in order. Do not skip steps.

## Golden rule: strict scope

Test **only** what the requirement says. Nothing more.
- No extra tests "just in case".
- No extra features or behaviour, even if they look good.
- If you think something is missing, **ask** (comment on the issue). Do not add it.

Extra tests cost review time and money, and they force code nobody asked for.

---

## Step 1: Read the requirement

- Read the whole issue or spec. Also read any parent issue it links to.
- Read the existing code and tests nearby. Copy their style.
- Write down anything unclear. **Do not guess.** Ask, or comment on the issue.
  If you must pick, pick the simplest option and write it in your PR.

## Step 2: List the behaviours

Write a plain list. One line per behaviour. Use this form:

> **When** <input or situation>, **then** <exact result>.

Example:
- When name is `"  foo  "`, then the saved name is `"foo"`.
- When name is blank, then validation fails on field `name`.

Each spec rule becomes one or more lines. If a rule gives no exact result, it is unclear. Go back to Step 1.

## Step 3: Add the edge cases

Use this list to find edge cases **inside** a stated requirement only.
Example: the spec says "name max 100 chars", so test 100 and 101.
If an edge case is not covered by any requirement, **do not test it. Ask instead.**

**Strings**
- empty `""`, whitespace only `"   "`, leading/trailing spaces
- exactly at the limit, one over the limit
- multibyte text (emoji, accents): count characters, not bytes
- special characters: quotes, `<script>`, newlines, null byte

**Numbers**
- 0, negative, max, one over max, one under min

**Optional fields**
- missing, `null`, empty string. Are they the same or different? The spec must say.

**Collections**
- empty, one item, many items, order of results

**Identity / uniqueness**
- duplicate value, duplicate with different case
- unknown id, malformed id

**Errors**
- several things wrong at once: are all errors reported?
- the error says *which* field and *why*

**State changes**
- create then read back
- update only changes what was given
- delete then read gives not found; delete twice
- timestamps: created stays the same, updated changes

**Boundaries between layers**
- bad input never causes a crash or a 500 error
- the database rule matches the code rule (both reject the same thing)

## Step 4: Plan the tests

Map each behaviour line to a test. Put the test in the right layer:

| layer | what it tests | real or fake? |
|---|---|---|
| unit | pure logic (validation, mapping) | no IO at all |
| storage | queries, constraints | **real database** |
| API | status code + response body | real app + real database |

Rules:
- Test each rule at the **lowest** layer that can see it.
- Test each rule **once** at the API layer too, if users can reach it there.
- Do not fake the database. Fakes hide real bugs.

## Step 5: Write each test

Each test:
1. **Name says the behaviour.** `create_rejects_blank_name`, not `test_create_2`.
2. **Arrange, Act, Assert.** Set up, do one thing, check results.
3. **One behaviour per test.** Several asserts are fine if they check that one behaviour.
4. **Assert exact values.** The field name, the error code, the status, the body field.
   - Bad: `assert!(result.is_err())`
   - Good: `assert_eq!(errors[0].field, "name")`
5. **Independent.** No shared state. No order dependency. Each test makes its own data.
6. **No sleeps, no real clock, no randomness** in what you assert.
7. **Readable data.** Use clear values like `"a".repeat(101)`, not magic strings.

Small helpers (build a test app, send a JSON request) are fine. Keep them simple and shared.

## Step 6: Make tests compile but fail

The code does not exist yet. So:
- Add the real **types and function signatures** the spec needs.
- Make the bodies fail on purpose (e.g. `todo!()`, `throw new Error("not implemented")`, or return 501).
- Run the tests. They must **compile** and **fail**. Old tests must still pass.

A test that passes now is suspicious. Either it tests nothing, or the behaviour already exists. Check it.

## Step 7: Check your own work

Go through this list. Fix anything that fails.

- [ ] Every behaviour line from Step 2 and Step 3 has a test.
- [ ] Every test maps back to a requirement. No extra tests that the spec does not ask for.
- [ ] No test only checks `is_ok`, `is_err`, `not null`, or "status is not 500".
- [ ] Each test would fail if the code was wrong in the way the test cares about.
      Ask: "What wrong code would still pass this test?" If you can think of one, make the assert stronger.
- [ ] No test is skipped, ignored, or commented out.
- [ ] Test names read like a list of rules.
- [ ] Tests run alone and in any order.

## Step 8: Report

In the PR description, add a table:

| requirement | test name(s) |
|---|---|
| name is trimmed | `create_trims_name` |

Also list:
- any choice you made where the spec was unclear
- any requirement you could not test, and why

---

## Stack notes

If the project uses Rust (axum, sqlx), read `rust.md` in this folder before writing tests.
