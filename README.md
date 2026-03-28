# hacli — Home Assistant CLI Reference

`hacli` is a Rust CLI for the Home Assistant REST API.

## Installation

### Quick Install (Linux/macOS)

```sh
curl -fsSL https://raw.githubusercontent.com/uintptr/hacli/main/scripts/install.sh | bash
```

This installs `hacli` to `~/.local/bin`.

### From Source

```sh
cargo install --git https://github.com/uintptr/hacli hacli
```

## Configuration

Credentials are resolved in this order (highest wins):

1. `--url` / `--token` flags
2. `HA_URL` / `HA_TOKEN` environment variables
3. `~/.config/hacli/config.toml`

```toml
# ~/.config/hacli/config.toml
url = "http://homeassistant.local:8123"
token = "your-long-lived-access-token"
```

## Global flags

These flags are available on every command:

| Flag | Env var | Default | Description |
|---|---|---|---|
| `--url <URL>` | `HA_URL` | config file | Home Assistant base URL |
| `--token <TOKEN>` | `HA_TOKEN` | config file | Long-lived access token |
| `--output` / `-o` | — | `json` | Output format: `json`, `table`, `plain` |

---

## hacli config

Manage the local configuration file. Does **not** require HA credentials.

### `hacli config init`

Interactively create or overwrite `~/.config/hacli/config.toml`.
Prompts for URL and token (token input is masked).

### `hacli config path`

Print the path to the config file.

---

## hacli api

API connectivity and Home Assistant system information.

### `hacli api ping`

Verify the API is accessible and the token is valid.

```
GET /api/
```

### `hacli api config`

Show the current Home Assistant configuration (components, timezone, location, units, version).

```
GET /api/config
```

### `hacli api error-log`

Print the current session error log as plain text.

```
GET /api/error_log
```

---

## hacli state

Entity state management.

### `hacli state list`

List all entity states.

```
GET /api/states
```

### `hacli state get <entity_id>`

Get the current state of a specific entity.

```
GET /api/states/<entity_id>
```

**Arguments:**

| Argument | Required | Description |
|---|---|---|
| `entity_id` | yes | Entity ID, e.g. `sensor.living_room_temperature` |

**Example:**
```sh
hacli state get sensor.living_room_temperature
hacli -o plain state get light.kitchen
```

### `hacli state set <entity_id> --state <value> [--attr KEY=VALUE ...]`

Create or update an entity state. Returns the new state object.

```
POST /api/states/<entity_id>
```

**Arguments:**

| Argument | Required | Description |
|---|---|---|
| `entity_id` | yes | Entity ID to create or update |
| `--state <value>` | yes | New state value (e.g. `on`, `23.5`, `unavailable`) |
| `--attr KEY=VALUE` | no, repeatable | Attribute key-value pair |

Attribute values are parsed as JSON first (so `128`, `true`, `null`, `[1,2]` become their native types). Anything that is not valid JSON is treated as a plain string.

**Examples:**
```sh
hacli state set sensor.my_sensor --state 42.5 --attr unit_of_measurement=°C
hacli state set input_boolean.test --state on --attr friendly_name="Test Switch"
```

### `hacli state delete <entity_id>`

Remove an entity from Home Assistant.

```
DELETE /api/states/<entity_id>
```

---

## hacli service

Service discovery and invocation.

### `hacli service list [domain]`

List all available services. Pass an optional domain to filter results.

```
GET /api/services
```

**Arguments:**

| Argument | Required | Description |
|---|---|---|
| `domain` | no | Filter to a specific domain, e.g. `light` |

**Examples:**
```sh
hacli service list
hacli service list light
hacli -o table service list switch
```

### `hacli service call <domain> <service> [--field KEY=VALUE ...] [--return-response]`

Call a service.

```
POST /api/services/<domain>/<service>
```

**Arguments:**

| Argument | Required | Description |
|---|---|---|
| `domain` | yes | Service domain, e.g. `light` |
| `service` | yes | Service name, e.g. `turn_on` |
| `--field KEY=VALUE` | no, repeatable | Service data field |
| `--return-response` | no | Include the service return value in output |

Field values follow the same JSON-first parsing as `--attr`.

**Examples:**
```sh
hacli service call light turn_on --field entity_id=light.living_room
hacli service call light turn_on --field entity_id=light.kitchen --field brightness=200
hacli service call climate set_temperature --field entity_id=climate.bedroom --field temperature=21.5
hacli service call script turn_on --field entity_id=script.good_morning --return-response
```

---

## hacli event

Event listing and firing.

### `hacli event list`

List all registered event types and their listener counts.

```
GET /api/events
```

### `hacli event fire <event_type> [--field KEY=VALUE ...]`

Fire a custom event, optionally with data.

```
POST /api/events/<event_type>
```

**Arguments:**

| Argument | Required | Description |
|---|---|---|
| `event_type` | yes | Event type name, e.g. `my_custom_event` |
| `--field KEY=VALUE` | no, repeatable | Event data field |

**Example:**
```sh
hacli event fire my_custom_event --field source=cli --field priority=1
```

---

## hacli history

Query historical state changes.

```
GET /api/history/period/<timestamp>
```

**Arguments:**

| Flag | Required | Description |
|---|---|---|
| `--entity-id <id>` | no | Filter to a specific entity |
| `--from <timestamp>` | no | Start time in ISO 8601 (defaults to 1 day ago) |
| `--to <timestamp>` | no | End time in ISO 8601 |
| `--minimal` | no | Only return `state` and `last_changed` fields |
| `--no-attributes` | no | Strip attributes from the response |
| `--significant-changes-only` | no | Only include entries where the state value changed |

**Examples:**
```sh
hacli history --entity-id sensor.temperature
hacli history --entity-id light.kitchen --from 2024-01-01T00:00:00+00:00 --to 2024-01-02T00:00:00+00:00
hacli history --entity-id sensor.power --minimal --significant-changes-only
```

---

## hacli logbook

Query the logbook.

```
GET /api/logbook/<timestamp>
```

**Arguments:**

| Flag | Required | Description |
|---|---|---|
| `--entity-id <id>` | no | Filter to a specific entity |
| `--from <timestamp>` | no | Start time in ISO 8601 |
| `--to <timestamp>` | no | End time in ISO 8601 |

**Example:**
```sh
hacli logbook --entity-id light.living_room --from 2024-01-01T00:00:00Z
```

---

## hacli calendar

Calendar entities and events.

### `hacli calendar list`

List all calendar entities.

```
GET /api/calendars
```

### `hacli calendar events <calendar_id> --start <timestamp> --end <timestamp>`

Fetch events from a calendar entity within a date range.

```
GET /api/calendars/<calendar_id>?start=...&end=...
```

**Arguments:**

| Argument | Required | Description |
|---|---|---|
| `calendar_id` | yes | Calendar entity ID, e.g. `calendar.my_calendar` |
| `--start <timestamp>` | yes | Range start (exclusive) in ISO 8601 |
| `--end <timestamp>` | yes | Range end (exclusive) in ISO 8601 |

**Example:**
```sh
hacli calendar events calendar.personal --start 2024-01-01T00:00:00Z --end 2024-02-01T00:00:00Z
```

---

## hacli template `<template>`

Render a Jinja2 template against live Home Assistant state.
Output format is ignored — the result is always printed as plain text.

```
POST /api/template
```

**Arguments:**

| Argument | Required | Description |
|---|---|---|
| `template` | yes | Jinja2 template string |

**Examples:**
```sh
hacli template "{{ states('sun.sun') }}"
hacli template "{{ state_attr('weather.home', 'temperature') }}°C"
hacli template "{% for s in states.light %}{{ s.entity_id }}: {{ s.state }}\n{% endfor %}"
```

---

## hacli check-config

Validate `configuration.yaml`. Requires the `config` integration to be loaded in Home Assistant.

```
POST /api/config/core/check_config
```

---

## KEY=VALUE parsing rules

Both `--field` (service/event data) and `--attr` (state attributes) use the same parser:

- The first `=` is the delimiter; subsequent `=` characters belong to the value.
- Values are parsed as JSON first: `true` → bool, `42` → integer, `3.14` → float, `null` → null, `[1,2]` → array, `{"a":1}` → object.
- Anything that is not valid JSON is treated as a plain string.

| Input | Rust type |
|---|---|
| `name=living room` | String |
| `brightness=200` | Number (integer) |
| `enabled=true` | Bool |
| `temperature=21.5` | Number (float) |
| `ids=["light.a","light.b"]` | Array |
| `meta={"source":"cli"}` | Object |
| `value=null` | Null |
