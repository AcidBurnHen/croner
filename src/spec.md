# croner Configuration Specification (`config.croner`)

## Overview

`croner` is a high-performance cron-like job runner written in Rust, evolved from the Python-based [PyCroner](https://github.com/AcidBurnHen/pycroner) project.\
While PyCroner used YAML for configuration, Croner is designed for **speed, stability, and local-first execution**.\
To meet these goals, we’ve **ditched YAML** in favor of a minimal, line-based configuration format optimized for quick parsing and atomic reloads.

## File Name

- Default: `config.croner`
- Override via CLI:
  ```
  croner --config=/path/to/my_config.croner
  ```

## Format Overview

- **INI-like** sections for each job.
- **Key/value** pairs per section.
- **No indentation, quoting, or YAML complexity**.
- Parsed in a single pass for maximum startup performance.

Example:

```
[job:my_first_job]
schedule = */5 * * * *
command = echo "Hello World"

[job:etl_process]
schedule = 0 2 * * *
command = python etl.py
fanout = 4

[job:multi_target_sync]
schedule = 15 3 * * *
command = python sync.py
fanout[] = --env=prod --sync=full
fanout[] = --env=dev --sync=partial
```

---

## Field Definitions

### `[job:<id>]`

- **Type**: Section header.
- **Required**: Yes.
- **Rules**:
  - `<id>` must be unique across the config.
  - Used internally for logging and tracking.

### `schedule`

- **Type**: String (5-field crontab syntax)
- **Required**: Yes.
- **Format**: `minute hour day month weekday`
  - Examples:
    - `* * * * *` → every minute
    - `0 0 * * *` → daily at midnight
    - `*/5 * * * *` → every 5 minutes
- Evaluated using Croner’s built-in parser (no `croniter` dependency).

### `command`

- **Type**: String
- **Required**: Yes.
- **Description**:
  - Shell command to execute.
  - Parsed into arguments using Croner’s **zero-dependency shell splitter**.
  - No shell expansion unless your command explicitly invokes a shell (`sh -c` / `cmd /c`).

### `fanout`

- **Type**: Integer or multiple `fanout[]` lines.
- **Required**: No.
- **Behavior**:
  - **Integer**: Run the same command N times in parallel.
  - ``** list**: Append each string to the base command and run separately.

#### Fanout as Integer

```
fanout = 3
```

- Runs:
  ```
  echo "Hello World"
  echo "Hello World"
  echo "Hello World"
  ```

#### Fanout as List

```
fanout[] = --env=prod --sync=full
fanout[] = --env=dev --sync=partial
```

- Runs:
  ```
  python sync.py --env=prod --sync=full
  python sync.py --env=dev --sync=partial
  ```

---

## Example `config.croner`

```
[job:index_articles]
schedule = */15 * * * *
command = python index.py
fanout = 4

[job:daily_etl]
schedule = 0 2 * * *
command = python etl.py
fanout[] = --source=internal --mode=full
fanout[] = --source=external --mode=delta

[job:ping]
schedule = * * * * *
command = python ping.py
```

---

## Notes

- All fields are case-sensitive.
- Jobs are scheduled with **sub-second precision** and minimal CPU overhead using a binary heap scheduler.
- Configuration reloads are **atomic** — invalid configs are rejected, and the running schedule is preserved.
- Fanout jobs are independent; failure in one does not affect the others.
- Commands are executed without invoking a shell unless explicitly configured.

