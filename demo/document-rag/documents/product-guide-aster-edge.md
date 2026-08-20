# Aster Edge Product Guide

**Version:** 3.4  
**Updated:** 2026-06-01

Aster Edge is a site-installed gateway that reads device telemetry, applies local rules, and securely synchronizes with Vortex Cloud.

## Core behavior

Aster polls supported devices every 15 seconds by default. It signs and batches observations before upload. If a site loses internet access, Aster writes events and commands to an encrypted local queue. On reconnect, it uploads queued telemetry in chronological order and reports a `backfill_complete` event to Vortex Cloud.

The local queue is capped at 72 hours of standard telemetry. If the queue is full, Aster preserves alarm and command-audit events and begins sampling ordinary telemetry at five-minute intervals. This does not affect local safety interlocks, which remain on the controller.

## Local rules

Operators may deploy approved threshold rules from Vortex Cloud. A local rule can create an alarm, attach a recommended runbook, or hold a non-safety command for operator review. Aster never autonomously changes inverter set points unless the site has the optional Closed Loop Automation entitlement and an approved site policy.

## Installation notes

Installers register each gateway against a single customer organization and site. The registration token expires after 30 minutes. Aster requires outbound HTTPS access to `ingest.vortexlab.example` and time synchronization via NTP.

See `api-integration-guide.md` for data payloads and `security-and-access-policy.md` for credential handling.
