# Incident Report: Northstar Alert Delay

**Date:** 2026-05-14  
**Severity:** SEV-2  
**Status:** Closed

## Summary

Northstar Energy received delayed high-temperature alerts for three battery sites between 09:12 and 10:03 UTC. No equipment damage occurred. Operators identified the condition through their local SCADA system and placed the affected units in a safe operating mode.

## Cause

On May 12, an alert-processing configuration change increased Pulse’s late-event watermark from 10 to 60 minutes for all organizations. The change was intended to reduce duplicate pages caused by one intermittent site. Aster Edge gateways at Northstar backfilled valid events after brief cellular outages; Pulse withheld their alert evaluation until the expanded watermark elapsed.

## Resolution and follow-up

The watermark was restored to 10 minutes at 10:03 UTC. We added organization-scoped configuration validation, a test fixture for delayed Aster events, and a dashboard showing alert-evaluation lag. The previous customer-specific workaround was removed.

Northstar received a written incident summary, a 30-day alerting credit, and weekly progress updates until the follow-up items were complete. See `q2-customer-success-notes.md` for the commercial commitments.
