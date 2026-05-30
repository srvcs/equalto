# srvcs-equalto

The comparison primitive of the srvcs.cloud distributed standard library.

Its single concern: **does `a` equal `b`?** It does not validate input itself —
it delegates "is this a number" to [`srvcs-isnumber`](https://github.com/srvcs/isnumber)
over HTTP, the single source of truth for that question, once per operand.
Equality is then computed on the integers (`a == b`).

If `srvcs-isnumber` is unreachable, `srvcs-equalto` reports itself **degraded
(503)** rather than guessing.

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/` | Service identity, concern, and dependency list |
| `POST` | `/` | Does `a` equal `b`? |
| `GET` | `/healthz` `/readyz` `/metrics` `/openapi.json` | srvcs service standard surface |

```sh
curl -s -X POST localhost:8080/ -H 'content-type: application/json' -d '{"a": 4, "b": 4}'
# {"a":4,"b":4,"result":true}
```

Responses:

- `200 {"a": a, "b": b, "result": bool}` — evaluated.
- `422` — an operand is not a number (per `srvcs-isnumber`) or not an integer.
- `503` — a dependency is unavailable.

## Dependencies

- [`srvcs-isnumber`](https://github.com/srvcs/isnumber) — input validation
  (called once per operand).

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `SRVCS_BIND_ADDR` | `0.0.0.0:8080` | Bind address |
| `SRVCS_ISNUMBER_URL` | `http://127.0.0.1:8081` | Base URL of `srvcs-isnumber` |
| `SRVCS_ENV` | `development` | Environment label for logs |
| `RUST_LOG` | `info,tower_http=info` | Tracing filter |

## Local checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Orchestration tests stand up a mock `srvcs-isnumber` in-process, so the suite
runs without the rest of the fleet. See
[`srvcs/platform`](https://github.com/srvcs/platform) for the shared standard.

> Note: the `cargoHash` in `flake.nix` is inherited from the template and must be
> refreshed with a `nix build` before the Nix gates pass.
