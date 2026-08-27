# AIONS Control Plane

Purpose: coordinate controlled operations across Windows, PowerShell, GitHub, GitHub Actions, Azure and Darkstar without bypassing Darkstar policy.

## Control flow

Request -> Session -> Capability -> Policy -> Orchestrator -> Provider -> External System -> Event -> Live Graph -> Audit

## Responsibilities

- **Darkstar**: policy, capabilities, orchestration boundary and event provenance.
- **PowerShell / Power Automate Desktop**: local Windows automation and adapters.
- **GitHub**: source, branches, commits, pull requests and Actions.
- **Azure**: remote compute, VM lifecycle and isolated environments.
- **Live Graph**: visualizes architecture, state transitions and active runs.
- **Round Table**: human and agent participants share a typed session and scoped capabilities.

## Provenance fields

Every automated operation should carry `request_id`, `run_id`, `principal_id`, `repo`, `branch`, `commit_sha`, `provider`, `started_at`, `finished_at` and `result`.

## Safety boundary

Automation adapters execute only commands approved by the Darkstar capability and policy layers. Providers do not make authorization decisions.

## Planned adapters

- Windows PowerShell
- Power Automate Desktop
- GitHub Actions
- Azure VM / network lifecycle
- Docker
- Kali-based Darkstar runtime
- Future Redox targets
