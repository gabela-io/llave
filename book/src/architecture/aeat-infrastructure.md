# AEAT Infrastructure

AEAT distributes its mobile API across multiple subdomains, each backed by separate WebSphere Application Server clusters.

## Subdomain Map

```mermaid
graph LR
    subgraph AEAT Infrastructure
        www2["www2.agenciatributaria.gob.es<br/><i>Primary gateway</i><br/>Session init, DNI auth,<br/>device management"]
        www12["www12.agenciatributaria.gob.es<br/><i>Registration services</i><br/>Registration state, SMS,<br/>SMS validation"]
        www6["www6.agenciatributaria.gob.es<br/><i>Activation services</i><br/>Device activation"]
    end

    Client["Mobile Client"] -->|"Step 0-1"| www2
    Client -->|"Step 2-4"| www12
    Client -->|"Step 5"| www6

    www2 -.->|"Redirect (302)"| www12
```

## Subdomain Roles

| Subdomain | Base URL | Role |
|-----------|----------|------|
| **www2** | `https://www2.agenciatributaria.gob.es` | Primary entry point. Handles session initialization (`ClaveStartingSv`), DNI/NIE authentication (`AutenticaDniNieContrasteh`), and most device management endpoints. |
| **www12** | `https://www12.agenciatributaria.gob.es` | Registration and SMS services. Handles registration state checks (`ClaveRequestStateSv`), SMS code requests (`ObtenerClaveMovilSMS`), and SMS validation (`ValidarClaveMovilSMS`). |
| **www6** | `https://www6.agenciatributaria.gob.es` | Device activation. Handles the final activation step (`ClaveActivateAuthenticationSv`). |
| **www1** | `https://www1.agenciatributaria.gob.es` | Browser-based operations, alternate for www12. |
| **sede** | `https://sede.agenciatributaria.gob.es` | AEAT sede (public website). |
| **pasarela** | `https://pasarela.clave.gob.es` | SAML identity gateway (DNIe/certificate auth). |

All endpoints live under the `/wlpl/MOVI-P24H/` path (mobile operations), except the DNI/NIE authentication which uses `/wlpl/BUCV-JDIT/` (identity verification subsystem).

## WebSphere Cluster Isolation

Each subdomain runs a separate WebSphere cluster with separate `JSESSIONID` namespaces. A `JSESSIONID` from www2 **must not** be sent to www6 -- doing so causes www6 to return its homepage HTML instead of processing the API request.

See [Cookie Management](../implementation-guide/cookie-management.md) for how cookies are propagated across subdomains, and [JSESSIONID Stripping](../implementation-guide/cookie-management.md#jsessionid-stripping) for the critical stripping requirement.
