# Subscription Payments Example

An on-chain subscription contract that charges a fixed fee per billing period.

```sh
soroban-forge new my-subscription --template subscription
```

## Features

- Subscriber pre-authorises recurring payments
- Owner can call `charge(subscriber)` once per period
- Subscriber can cancel at any time

## Key Functions

| Function | Description |
|----------|-------------|
| `initialize(merchant, token, amount, interval)` | Configure the plan |
| `subscribe(subscriber)` | Opt in and pay the first period |
| `charge(subscriber)` | Collect a period once it is due |
| `cancel(subscriber)` | Terminate the subscription |

The subscriber pre-authorises payments with a token `approve` allowance, so the
contract never takes custody of their funds. See
[docs/templates.md](../../docs/templates.md#subscription).
