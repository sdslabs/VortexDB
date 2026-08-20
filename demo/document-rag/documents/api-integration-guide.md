# Vortex Cloud API Integration Guide

**API version:** v2  
**Updated:** 2026-05-18

The Vortex Cloud API lets customers read site telemetry, acknowledge alarms, and export reports. API access is available on Growth and Enterprise plans.

## Authentication

Use an organization-scoped service account with a short-lived OAuth token. Tokens expire after one hour. Service accounts receive only the scopes explicitly assigned to them; use `telemetry:read`, `alarms:write`, and `reports:read` rather than a broad administrative scope.

## Key endpoints

`GET /v2/sites/{site_id}/telemetry?from=&to=` returns normalized readings. The maximum query window is 31 days.

`GET /v2/alarms` accepts filters for status, severity, site, and tag. Alarm timestamps are in UTC.

`POST /v2/alarms/{alarm_id}/acknowledgements` records the named operator, timestamp, and optional note. It does not resolve the alarm; resolution requires the `alarms:resolve` scope.

`POST /v2/exports` creates an asynchronous CSV or Parquet export. Exports are retained for 24 hours and are listed in the audit log.

## Rate limits and backfill

The default limit is 120 requests per minute per service account. During an Aster connectivity recovery, telemetry can arrive up to 72 hours late; clients should order records by `observed_at`, not API receipt time.

Never embed service-account secrets in Aster configuration files or source repositories.
