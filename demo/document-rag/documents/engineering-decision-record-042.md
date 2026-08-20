# ADR-042: Keep a 10-Minute Alert Watermark

**Date:** 2026-05-28  
**Status:** Accepted

## Context

Pulse must handle delayed telemetry from Aster Edge without causing duplicate or misleading pages. A May incident demonstrated that a globally expanded 60-minute watermark made operational alerts unacceptably late for battery customers.

## Decision

The default alert watermark remains 10 minutes. Any exception longer than 15 minutes must be organization-scoped, approved by Reliability Engineering, have an expiry date, and be covered by a replay test using delayed events.

Events received after the watermark are marked as backfill. They update historical charts and reports but do not reopen a resolved alarm automatically. Operators can manually review a backfill event from the alarm timeline.

## Consequences

Some intermittently connected sites may continue to create duplicate candidate alerts, which Pulse deduplicates using rule and event identity. This is preferable to silently delaying high-severity alerts across unrelated customers.
