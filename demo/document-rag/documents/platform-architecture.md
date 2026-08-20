# Platform Architecture

**Updated:** 2026-06-10  
**Audience:** Engineering and Security

Vortex Cloud separates ingestion, operational storage, alert evaluation, and customer-facing applications.

1. **Ingress** validates device identity, schema, and message signatures.
2. **Stream processing** enriches events with site metadata and writes immutable raw telemetry.
3. **Pulse** evaluates anomaly models and deterministic alert rules.
4. **Operations API** serves the web application, reports, and customer integrations.

Raw telemetry is encrypted at rest and logically partitioned by organization ID. The web application uses the Operations API; it does not directly query the telemetry store. Customer data is replicated within the selected hosting region for durability.

## Resilience

Ingress accepts delayed events from Aster Edge and deduplicates them using gateway ID, sequence number, and observed timestamp. Pulse uses a 10-minute watermark for normal alert evaluation; events older than the watermark are marked as backfill and may update reports, but do not automatically reopen a resolved incident.

## Known trade-off

The watermark avoids duplicate pages during connectivity recovery, but it can delay alert classification for intermittently connected sites. The Northstar incident in May 2026 exposed a configuration error in this boundary; details are in `incident-2026-05-northstar.md`.
