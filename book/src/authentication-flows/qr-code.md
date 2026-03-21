# QR Code Authentication

QR authentication is a two-step process.

## Flow

```
1. ClaveMovilQrSv  ->  Submit QR code (www2)
   Fields: device_id, NIF, valor_qr, OS info
   Response: { status: "OK", respuesta: { tokenClaveMovil, codigoIdP, ... } }

2. ClaveAuthenticateSv  ->  Confirm the authentication (www2)
   Fields: device_id, user_password, NIF, tokenClaveMovil, codigoIdP
   (Same as the Clave Movil confirm step)
```

The QR value is obtained by scanning a QR code displayed on a government service's login page. Step 1 retrieves the authentication token, step 2 confirms it.

## Details

- Step 1 (`ClaveMovilQrSv`) takes the raw QR string scanned by the camera as `valor_qr`.
- Step 2 is identical to the [Clave Movil confirm step](./clave-movil.md#confirm-flow) -- it uses the `tokenClaveMovil` and `codigoIdP` returned by step 1.
