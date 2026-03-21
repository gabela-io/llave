# Token Activation

Device activation via a pre-existing token (e.g. from a previous registration method).

## Flow

```
1. ObtenerClaveMovil  ->  Get activation page (www12, GET)
2. ValidarClaveMovil  ->  Validate token (www12)
   Fields: tokenClaveMovil
```

This flow is used when the user already has a `tokenClaveMovil` (e.g. from an in-person registration or letter-based activation) and needs to link it to a device.
