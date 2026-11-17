# Doubles-to-Go: agent guide

Pre-order app for street-food vendors in Trinidad. One deployable Node service plus static pages.
The design is in `arc42.md`. The code is in `app/`.

## Read first
- Constraints C1 to C5: `arc42.md` section 2
- Block responsibilities and contracts: `arc42.md` section 5 (5.4 PlaceOrder, 5.5 MarkPaid, 5.6 HandleWebhook)
- Why things are the way they are: `arc42.md` section 9 (ADR-0001 to ADR-0006)
- Where each block lives in the code: `app/README.md`

## Rules
- Card data never enters our code, database or logs. Payment happens on the provider's hosted page (C1, ADR-0002).
- Every write request carries an idempotency key. A repeat returns the stored result (ADR-0005).
- The Payment Adapter and the Notification Adapter never touch the database. They reach the Order Service through the port they are given.
- Only `app/src/order-service/order-repository.js` contains SQL.
- An unknown payment result is never a failure. Keep the order PendingPayment and check again with backoff.
- Log ids and the correlation id. Never log names or phone numbers.
- Use Node built-ins only. Propose an ADR before adding a dependency.

## Do this, not that
- Call the provider through `payment-adapter.js`, not with `fetch` from elsewhere.
- Settle payments through the queue, not inside the webhook request (ADR-0003).
- Record a change of mind as a new ADR that supersedes the old one, never by editing it.

## Commands (run in `app/`)
- `npm test`: all tests, including the architecture rules and the arc42 checks
- `npm run docs:check`: consistency of `arc42.md`
- `npm run simulate`: scenario Q1 with 1,000 dropped connections
- `npm start`: http://localhost:3000 for customers, `/vendor/` and `/demo/`

## When you change behaviour
Update `arc42.md` in the same change (a contract, a scenario or an ADR), then run `npm test` and `npm run docs:check`.
