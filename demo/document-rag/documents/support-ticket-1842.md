# Support Ticket 1842: Missing Telemetry After Storm

**Customer:** Rivermark Solar  
**Opened:** 2026-06-03  
**Status:** Resolved

Rivermark reported a gap in inverter telemetry after a regional storm. Support verified that the Aster gateway remained powered but had no cellular route from 02:18 to 06:47 local time.

After connectivity returned, Aster uploaded the queued readings. The customer dashboard showed a temporary gap because its selected view was sorted by ingestion time; engineering confirmed the records existed when queried by `observed_at`.

Support advised Rivermark to use the “event time” option in reporting and linked the API integration guide. No data was lost. The case also prompted a UI improvement request to label backfilled data more clearly.
