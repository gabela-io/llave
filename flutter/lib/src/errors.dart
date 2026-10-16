import '../l10n/app_localizations.dart';
import 'llave_bridge.dart';

/// The message to show for a failed [result].
///
/// Known error kinds get a localised message. A server `KO` shows the AEAT
/// server's own message, as the official app does. Anything else falls back
/// to the core's (English) error text.
String friendlyError(AppLocalizations l10n, LlaveApiResult result) {
  final fallback = result.serverMessage ?? result.error ?? l10n.unknownError;
  return switch (result.errorCode) {
    'http' => l10n.errorNoConnection,
    'no_session' => l10n.errorNoSession,
    'device_not_active' => l10n.errorDeviceNotActive,
    'not_registered' => l10n.errorNotRegistered,
    'no_registered_phone' => l10n.errorNoRegisteredPhone,
    'app_version_outdated' => l10n.errorAppOutdated,
    'invalid_qr' => l10n.errorInvalidQr,
    'no_pending_pin' => l10n.pinNoPending,
    'dni_auth_failed' => l10n.errorDniData,
    'invalid_nif' => l10n.errorInvalidNif,
    'html_response' || 'json' || 'too_many_redirects' => l10n.errorUnexpectedResponse,
    'api' => result.serverMessage ?? l10n.errorUnexpectedResponse,
    // The IP block keeps its English text: `isIpRateLimited` matches it.
    'ip_rate_limited' => result.error ?? fallback,
    _ => fallback,
  };
}
