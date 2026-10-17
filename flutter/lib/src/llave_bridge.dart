import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';

// Hand-written dart:ffi bindings to the C ABI of llave-core-ffi.
import 'rust/api/api.dart' as ffi;

// ---------------------------------------------------------------------------
// Dart-side models of the FFI results, shared by the native and mock bridges.
// ---------------------------------------------------------------------------

class LlaveSession {
  final String deviceId;
  final String nif;
  final String createdAt;

  const LlaveSession({
    required this.deviceId,
    required this.nif,
    required this.createdAt,
  });
}

class LlaveStatus {
  final bool active;
  final String? nif;
  final String? deviceId;
  final String? createdAt;

  const LlaveStatus({
    required this.active,
    this.nif,
    this.deviceId,
    this.createdAt,
  });

  const LlaveStatus.inactive()
      : active = false,
        nif = null,
        deviceId = null,
        createdAt = null;
}

class LlaveApiResult {
  final bool ok;
  final String data;

  /// English error text, for logs.
  final String? error;

  /// Stable error kind from the core (`no_pending_pin`, `api`, …), used to
  /// show a localised message (see `friendlyError`).
  final String? errorCode;

  /// The AEAT server's own message for a `KO`.
  final String? serverMessage;

  const LlaveApiResult({
    required this.ok,
    required this.data,
    this.error,
    this.errorCode,
    this.serverMessage,
  });

  /// A failure from an exception thrown on the Dart side (for example the
  /// native library), rather than an error reported by the core.
  factory LlaveApiResult.exception(Object e) =>
      LlaveApiResult(ok: false, data: '', error: e.toString(), errorCode: 'exception');

  factory LlaveApiResult.fromFfi(ffi.FfiApiResult r) => LlaveApiResult(
        ok: r.ok,
        data: r.data,
        error: r.error,
        errorCode: r.errorCode,
        serverMessage: r.serverMessage,
      );
}

class LlaveNifCheckResult {
  final String nif;
  final String status;
  final String responseJson;

  const LlaveNifCheckResult({
    required this.nif,
    required this.status,
    required this.responseJson,
  });
}

// ---------------------------------------------------------------------------
// Bridge service interface.
//
// RealLlaveBridge calls the Rust core; MockLlaveBridge fakes it for UI work
// and tests. `bridgeTypeProvider` picks one.
// ---------------------------------------------------------------------------

abstract class LlaveBridge {
  /// Initialise the Rust core. [sessionJson] is the previously-saved session
  /// from secure storage, or `null` on first launch.
  Future<void> initCore(String? sessionJson);

  /// Export the current session as JSON for secure storage persistence.
  String? exportSession();

  /// Reveal the pending PIN: `data` is `{pin, time_to_live_seconds, nif}`;
  /// `time_to_live_seconds` is empty when the server sent no validity.
  Future<LlaveApiResult> requestPin();
  LlaveStatus getStatus();
  Future<LlaveNifCheckResult> checkNif(String? nif);
  Future<LlaveApiResult> getMyData();
  Future<LlaveApiResult> getHistory();
  Future<LlaveApiResult> getPendingRequests();
  Future<LlaveApiResult> confirmRequest(String token, String idpCode);
  Future<LlaveApiResult> rejectRequest(String token, String idpCode);
  Future<LlaveApiResult> qrAuthenticate(String value);
  Future<LlaveApiResult> deactivate();
  /// Phase 1 of activation. On a date-of-birth challenge the result fails
  /// with `data = {"challenge": …}`; call again with the date of birth as
  /// [fecha] and that [challenge].
  Future<LlaveApiResult> dniAuthenticate(String nif, String fecha, String soporte,
      {String challenge = ''});
  Future<LlaveApiResult> dniCompleteActivation(String nif, String cookiesJson,
      String timestampAltaSms, String tokenClaveMovilSms, String smsPin);
  bool logout();
  String validateNif(String nif);

  /// llave's version and the Cl@ve app version it matches.
  String version();

  /// Document field needed for [nif]: `validity_date`, `birth_date` or
  /// `support_number`. Throws when the NIF is invalid.
  String dniInput(String nif);

  /// Encrypt the current in-memory session with a PIN.
  String encryptSession(String pin);

  /// Decrypt a sealed blob and load the session into Rust core.
  bool decryptAndLoadSession(String sealed, String pin);

  /// Re-encrypt the session with a new PIN (requires old PIN).
  String changeSessionPin(String sealed, String oldPin, String newPin);

  /// Set or clear the proxy URL for all subsequent API calls.
  void setProxy(String? url);
}

/// Mock implementation that simulates the Rust bridge for UI development.
class MockLlaveBridge implements LlaveBridge {
  LlaveSession? _session;

  @override
  Future<void> initCore(String? sessionJson) async {
    // In mock mode, no Rust core to initialise.
  }

  @override
  String? exportSession() {
    // Mock: no real session data to export.
    return null;
  }

  @override
  Future<LlaveApiResult> requestPin() async {
    _requireSession();
    await Future<void>.delayed(const Duration(milliseconds: 800));
    final pin = (100000 + (DateTime.now().microsecond % 900000)).toString();
    return LlaveApiResult(
      ok: true,
      data: jsonEncode({'pin': pin, 'time_to_live_seconds': '180', 'nif': _session!.nif}),
    );
  }

  @override
  LlaveStatus getStatus() {
    if (_session == null) return const LlaveStatus.inactive();
    return LlaveStatus(
      active: true,
      nif: _session!.nif,
      deviceId: _session!.deviceId,
      createdAt: _session!.createdAt,
    );
  }

  @override
  Future<LlaveNifCheckResult> checkNif(String? nif) async {
    await Future<void>.delayed(const Duration(milliseconds: 500));
    final n = nif ?? _session?.nif ?? 'unknown';
    return LlaveNifCheckResult(
      nif: n,
      status: 'OK',
      responseJson: '{"activated": true}',
    );
  }

  @override
  Future<LlaveApiResult> getMyData() async {
    _requireSession();
    await Future<void>.delayed(const Duration(milliseconds: 600));
    return const LlaveApiResult(
      ok: true,
      data: '{"nif":"12345678Z","email":"mock@example.com","numTelefono":"***1234","nivelRegistro":"AVANZADO"}',
    );
  }

  @override
  Future<LlaveApiResult> getHistory() async {
    _requireSession();
    await Future<void>.delayed(const Duration(milliseconds: 600));
    return const LlaveApiResult(ok: true, data: '{"operations":[]}');
  }

  @override
  Future<LlaveApiResult> getPendingRequests() async {
    _requireSession();
    await Future<void>.delayed(const Duration(milliseconds: 400));
    // Same shape as the real bridge: the pending `peticion` or null.
    return const LlaveApiResult(ok: true, data: 'null');
  }

  @override
  Future<LlaveApiResult> confirmRequest(String token, String idpCode) async {
    _requireSession();
    await Future<void>.delayed(const Duration(milliseconds: 500));
    return const LlaveApiResult(ok: true, data: '{"confirmed":true}');
  }

  @override
  Future<LlaveApiResult> rejectRequest(String token, String idpCode) async {
    _requireSession();
    await Future<void>.delayed(const Duration(milliseconds: 500));
    return const LlaveApiResult(ok: true, data: '{"rejected":true}');
  }

  @override
  Future<LlaveApiResult> qrAuthenticate(String value) async {
    _requireSession();
    await Future<void>.delayed(const Duration(milliseconds: 500));
    return const LlaveApiResult(
      ok: true,
      data: '{"peticion":{"tokenClaveMovil":"mock-token","codigoIdP":"4821","organismo":"Agencia Tributaria"}}',
    );
  }

  @override
  Future<LlaveApiResult> deactivate() async {
    _requireSession();
    await Future<void>.delayed(const Duration(milliseconds: 500));
    _session = null;
    return const LlaveApiResult(
      ok: true,
      data: '{"status":"OK","message":"Device deactivated"}',
    );
  }

  @override
  Future<LlaveApiResult> dniAuthenticate(String nif, String fecha, String soporte,
      {String challenge = ''}) async {
    await Future<void>.delayed(const Duration(milliseconds: 800));
    return const LlaveApiResult(
      ok: true,
      data: '{"movil":"***1234","hora_peticion":"12:00","timestamp_alta_sms":"mock_ts","token_clave_movil_sms":"mock_token","cookies_json":"[]","registrado":"S","nivel_registro":"A","telefono":"S"}',
    );
  }

  @override
  Future<LlaveApiResult> dniCompleteActivation(String nif, String cookiesJson,
      String timestampAltaSms, String tokenClaveMovilSms, String smsPin) async {
    await Future<void>.delayed(const Duration(milliseconds: 800));
    final validated = validateNif(nif);
    _session = LlaveSession(
      deviceId: 'mock-dni-${DateTime.now().millisecondsSinceEpoch}',
      nif: validated,
      createdAt: DateTime.now().toIso8601String(),
    );
    return LlaveApiResult(
      ok: true,
      data: '{"device_id":"${_session!.deviceId}","nif":"$validated"}',
    );
  }

  @override
  bool logout() {
    _session = null;
    return true;
  }

  @override
  String validateNif(String nif) {
    final trimmed = nif.trim().toUpperCase();
    if (trimmed.length < 8 || trimmed.length > 9) {
      throw ArgumentError('NIF must be 8-9 characters');
    }
    final first = trimmed[0];
    if (!RegExp(r'[0-9KLMXYZ]').hasMatch(first)) {
      throw ArgumentError('NIF must start with 0-9, K, L, M, X, Y or Z');
    }
    return trimmed;
  }

  @override
  String version() => 'mock';

  @override
  String dniInput(String nif) {
    // Mirrors llave_core::config::required_dni_input for UI development.
    return switch (validateNif(nif)[0]) {
      'K' || 'L' || 'M' => 'birth_date',
      'X' || 'Y' || 'Z' => 'support_number',
      _ => 'validity_date',
    };
  }

  @override
  String encryptSession(String pin) => 'mock_sealed_blob';

  @override
  bool decryptAndLoadSession(String sealed, String pin) {
    if (pin.length < 4) throw ArgumentError('Wrong PIN');
    _session = LlaveSession(
      deviceId: 'mock-device',
      nif: 'mock-nif',
      createdAt: DateTime.now().toIso8601String(),
    );
    return true;
  }

  @override
  String changeSessionPin(String sealed, String oldPin, String newPin) =>
      'mock_resealed_blob';

  @override
  void setProxy(String? url) {
    // No-op in mock mode.
  }

  void _requireSession() {
    if (_session == null) {
      throw StateError('No active session. Activate device first.');
    }
  }
}

// ---------------------------------------------------------------------------
// Real bridge calling into the Rust core through the C ABI.
// ---------------------------------------------------------------------------

class RealLlaveBridge implements LlaveBridge {
  @override
  Future<void> initCore(String? sessionJson) async {
    await ffi.initCore(sessionJson: sessionJson);
  }

  @override
  String? exportSession() => ffi.exportSession();

  @override
  Future<LlaveApiResult> requestPin() async =>
      LlaveApiResult.fromFfi(await ffi.requestPin());

  @override
  LlaveStatus getStatus() {
    final s = ffi.getStatus();
    return LlaveStatus(
      active: s.active,
      nif: s.nif,
      deviceId: s.deviceId,
      createdAt: s.createdAt,
    );
  }

  @override
  Future<LlaveNifCheckResult> checkNif(String? nif) async {
    final r = await ffi.checkNif(nif: nif);
    return LlaveNifCheckResult(
      nif: r.nif,
      status: r.status,
      responseJson: r.responseJson,
    );
  }

  @override
  Future<LlaveApiResult> getMyData() async {
    final r = await ffi.getMyData();
    return LlaveApiResult.fromFfi(r);
  }

  @override
  Future<LlaveApiResult> getHistory() async {
    final r = await ffi.getHistory();
    return LlaveApiResult.fromFfi(r);
  }

  @override
  Future<LlaveApiResult> getPendingRequests() async {
    final r = await ffi.getPendingRequests();
    return LlaveApiResult.fromFfi(r);
  }

  @override
  Future<LlaveApiResult> confirmRequest(String token, String idpCode) async {
    final r = await ffi.confirmRequest(token: token, idpCode: idpCode);
    return LlaveApiResult.fromFfi(r);
  }

  @override
  Future<LlaveApiResult> rejectRequest(String token, String idpCode) async {
    final r = await ffi.rejectRequest(token: token, idpCode: idpCode);
    return LlaveApiResult.fromFfi(r);
  }

  @override
  Future<LlaveApiResult> qrAuthenticate(String value) async {
    final r = await ffi.qrAuthenticate(value: value);
    return LlaveApiResult.fromFfi(r);
  }

  @override
  Future<LlaveApiResult> deactivate() async {
    final r = await ffi.deactivate();
    return LlaveApiResult.fromFfi(r);
  }

  @override
  Future<LlaveApiResult> dniAuthenticate(String nif, String fecha, String soporte,
      {String challenge = ''}) async {
    final r = await ffi.dniAuthenticate(
        nif: nif, fecha: fecha, soporte: soporte, challenge: challenge);
    return LlaveApiResult.fromFfi(r);
  }

  @override
  Future<LlaveApiResult> dniCompleteActivation(String nif, String cookiesJson,
      String timestampAltaSms, String tokenClaveMovilSms, String smsPin) async {
    final r = await ffi.dniCompleteActivation(
        nif: nif,
        cookiesJson: cookiesJson,
        timestampAltaSms: timestampAltaSms,
        tokenClaveMovilSms: tokenClaveMovilSms,
        smsPin: smsPin);
    return LlaveApiResult.fromFfi(r);
  }

  @override
  bool logout() => ffi.logout();

  @override
  String validateNif(String nif) => ffi.validateNif(nif: nif);

  @override
  String version() => ffi.version();

  @override
  String dniInput(String nif) => ffi.dniInput(nif: nif);

  @override
  String encryptSession(String pin) => ffi.encryptSession(pin: pin);

  @override
  bool decryptAndLoadSession(String sealed, String pin) =>
      ffi.decryptAndLoadSession(sealed: sealed, pin: pin);

  @override
  String changeSessionPin(String sealed, String oldPin, String newPin) =>
      ffi.changeSessionPin(sealed: sealed, oldPin: oldPin, newPin: newPin);

  @override
  void setProxy(String? url) => ffi.setProxy(url: url);
}

// ---------------------------------------------------------------------------
// Riverpod providers
// ---------------------------------------------------------------------------

enum BridgeType { mock, real }

/// Which bridge implementation to use. Toggle from DevScreen.
final bridgeTypeProvider = StateProvider<BridgeType>((ref) => BridgeType.real);

/// Single bridge instance shared across the app, rebuilt when [bridgeTypeProvider] changes.
final llaveBridgeProvider = Provider<LlaveBridge>((ref) {
  return switch (ref.watch(bridgeTypeProvider)) {
    BridgeType.mock => MockLlaveBridge(),
    BridgeType.real => RealLlaveBridge(),
  };
});
