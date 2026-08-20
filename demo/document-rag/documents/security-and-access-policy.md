# Security and Access Policy

**Effective:** 2026-06-15  
**Policy owner:** Security Engineering

## Access principles

Vortex uses least privilege, organization isolation, and time-bounded elevated access. Employees authenticate with SSO and phishing-resistant MFA. Production access is granted through named roles and logged.

Contractors may access staging systems and sanitized support reproductions. They may not access production telemetry, production databases, customer exports, or incident channels containing customer data unless the Chief Information Security Officer grants a documented, time-limited exception.

## Customer data retention

Raw telemetry retention is controlled by the customer plan: Starter retains 30 days, Growth retains 90 days, and Enterprise retains 365 days by default. Enterprise customers can purchase an archival extension of up to seven years. Aggregated monthly metrics are retained for the life of an active account plus 12 months.

This policy supersedes the draft retention language in the January planning notes. Legal holds suspend deletion for the data in scope.

## Incident handling

Suspected unauthorized access must be reported to `security@vortexlab.example` and the on-call reliability engineer within one hour. Security Engineering coordinates investigation, customer notification, and required regulatory reporting.
