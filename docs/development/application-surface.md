# Application Surface (Implementation Snapshot)

## Intent

Document the currently implemented application surface so development and QA can
work from what exists in code today.

## Snapshot

- Reviewed against repository state on 2026-02-05.
- Scope: frontend route wiring, HTTP endpoints, and service integration points.

## Frontend route coverage

| Route | Implementation status | Data source |
| --- | --- | --- |
| `/dashboard` | UI implemented with static sample data | none |
| `/agents` | UI implemented with static sample data | none |
| `/connections` | API-backed | Integration Service (`/connections`) |
| `/integrations` | UI implemented with static sample data | none |
| `/integrations/:id/edit` | Canvas UI implemented (local state only) | none |
| `/data-hub` | UI implemented with static sample data | none |
| `/assets` | API-backed | Asset Service (`/assets*`) + Integration Service discovery endpoints |
| `/operations` | UI implemented with static sample data | none |
| `/operations/alerts` | UI implemented with static sample data | none |
| `/operations/incidents` | UI implemented with static sample data | none |
| `/operations/playbooks*` | API-backed workflow present | Integration Service playbook endpoints |
| `/governance/*` | UI implemented with static sample data | none |
| `/rationalization/*` | UI implemented with static sample data | none |
| `/settings` | UI implemented | none |

Notes:

- A global AI chat drawer is implemented and currently returns simulated
  responses in the browser.
- Sidebar includes `/ai`, but no route is registered for `/ai` in `App.tsx`.

## Backend HTTP APIs

### API Gateway (`:8081`)

Public endpoints:

- `GET /health`
- `GET /ready`

Protected base path: `/api/v1` (JWT bearer auth + tenant context middleware)

- Agents:
  - `GET /agents`
  - `GET /agents/{agentID}`
  - `DELETE /agents/{agentID}`
- Connections:
  - `GET /connections`
  - `POST /connections`
  - `GET /connections/{connectionID}`
  - `PUT /connections/{connectionID}`
  - `DELETE /connections/{connectionID}`
  - `POST /connections/{connectionID}/test`
- Integrations:
  - `GET /integrations`
  - `POST /integrations`
  - `GET /integrations/{integrationID}`
  - `PUT /integrations/{integrationID}`
  - `DELETE /integrations/{integrationID}`
  - `POST /integrations/{integrationID}/run`
  - `GET /integrations/{integrationID}/runs`
- Runs:
  - `GET /runs/{runID}`
  - `POST /runs/{runID}/cancel`
  - `GET /runs/{runID}/logs`
- Users (admin role):
  - `GET /users`
  - `POST /users`
  - `GET /users/{userID}`
  - `PUT /users/{userID}`
  - `DELETE /users/{userID}`

### Integration Service (`:8082`)

Public endpoints:

- `GET /health`
- `GET /ready`

Tenant-scoped endpoints (currently using optional tenant middleware in dev):

- Integrations and runs:
  - `GET /integrations`
  - `POST /integrations`
  - `GET /integrations/:id`
  - `POST /integrations/:id/run`
  - `GET /runs/:id`
  - `POST /runs/:id/cancel`
- Connections:
  - `GET /connections`
  - `POST /connections`
  - `GET /connections/:id`
  - `PUT /connections/:id`
  - `DELETE /connections/:id`
  - `POST /connections/:id/test`
- Discovery:
  - `POST /discovery/run`
  - `GET /discovery/runs`
  - `POST /dev/mock-discovery` (dev/local testing)
- Operations playbooks:
  - `GET /playbooks`
  - `POST /playbooks`
  - `GET /playbooks/:id`
  - `PUT /playbooks/:id`
  - `DELETE /playbooks/:id`
  - `POST /playbooks/:id/run`
  - `GET /playbooks/:id/runs`
  - `GET /playbook-runs/:id`
  - `POST /playbook-runs/:id/approve`
  - `POST /playbook-runs/:id/reject`

### Asset Service (`:8084`)

Public endpoints:

- `GET /health`
- `GET /ready`

Asset and relationship APIs:

- Assets:
  - `GET /assets`
  - `POST /assets`
  - `GET /assets/:id`
  - `PUT /assets/:id`
  - `DELETE /assets/:id`
  - `GET /assets/search`
- Relationships:
  - `GET /relationships`
  - `POST /relationships`
  - `DELETE /relationships/:id`
  - `GET /assets/:id/relationships`
- Graph:
  - `GET /graph/neighbors/:id`
  - `GET /graph/path`
  - `GET /graph/subgraph/:id`
- Impact analysis:
  - `GET /impact/:id`
  - `GET /impact/:id/downstream`
  - `GET /impact/:id/upstream`
  - Current implementation returns `501 Not Implemented`.

## Frontend API base behavior

The frontend currently uses a single base URL via `VITE_API_URL` (default:
`http://localhost:8082`).

Because implemented endpoints are split across services, local development
currently has two patterns:

1. Service-focused testing by pointing `VITE_API_URL` at one service at a time.
2. Running a local reverse proxy that maps Integration Service and Asset Service
   routes behind one origin.

## Known implementation gaps

- API Gateway does not yet proxy the Integration Service playbook/discovery
  routes or Asset Service routes.
- API Gateway rate limiting middleware is a placeholder (no Redis-backed
  enforcement yet).
- Some API Gateway operations are placeholders (`connection test`, integration
  dispatch/cancel task propagation).
- The web app has substantial UI surface not yet connected to backend data
  (dashboard, governance, rationalization, most operations pages).
- Playbook frontend client paths are namespaced under `/integrations/*` while
  Integration Service routes are rooted at `/playbooks*` and
  `/playbook-runs*`; this requires path rewriting or alignment.
