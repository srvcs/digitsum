# srvcs-digitsum

Number theory microservice for srvcs.cloud: the **sum of the decimal digits** of
an integer.

This is a **leaf** service. It depends on no other service and does all of its
work as local `i64` arithmetic.

## Concern

Given an integer `n`, take its magnitude `|n|` and sum its decimal digits. The
sign is ignored.

```
digitsum(123)  -> 6     (1 + 2 + 3)
digitsum(-49)  -> 13    (4 + 9, sign dropped)
digitsum(0)    -> 0
```

## API

### `GET /` — identity

```json
{
  "service": "srvcs-digitsum",
  "concern": "number theory: sum of decimal digits",
  "depends_on": []
}
```

### `POST /` — evaluate

Request:

```json
{ "value": 123 }
```

Response `200`:

```json
{ "value": 123, "result": 6 }
```

| Status | Meaning                              |
| ------ | ------------------------------------ |
| `200`  | digit sum computed                   |
| `422`  | `value` is missing or not an integer |
| `500`  | internal error                       |

Whole-valued floats (`123.0`) are accepted; genuinely fractional values
(`12.3`) are rejected with `422`.

## Local checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The OpenAPI document is committed at `openapi.json` and checked in CI. Regenerate
it with:

```sh
UPDATE_OPENAPI=1 cargo test --test openapi_snapshot
```

See [`srvcs/platform`](https://github.com/srvcs/platform) for the shared service
standard and CI workflow.
