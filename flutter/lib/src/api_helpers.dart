import 'dart:async';
import 'dart:convert';

import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../l10n/app_localizations.dart';

import 'auth_provider.dart';
import 'errors.dart';
import 'ip_rate_limit.dart';
import 'llave_bridge.dart';

/// Safely decode a JSON string, returning `null` on failure instead of
/// throwing a [FormatException].
dynamic tryJsonDecode(String data) {
  try {
    return jsonDecode(data);
  } on FormatException catch (e) {
    debugPrint('JSON decode failed: $e');
    return null;
  }
}

/// Wraps the common FFI API-call boilerplate shared by most screens:
///
/// 1. Calls [apiCall] and checks `result.ok`.
/// 2. On failure, sets [setError] and shows an IP-rate-limit banner when
///    applicable.
/// 3. On success, safe-decodes JSON via [tryJsonDecode] and forwards the
///    parsed value to [onData].
/// 4. Catches exceptions with the same rate-limit handling.
/// 5. Always calls `setLoading(false)` in a `finally` block.
Future<void> handleApiCall({
  required Future<LlaveApiResult> Function() apiCall,
  required void Function(bool) setLoading,
  required void Function(String?) setError,
  required BuildContext context,
  FutureOr<void> Function(dynamic decoded)? onData,
}) async {
  // Resolve localisations before the async gap so the context is not used
  // after it may have been unmounted.
  final l10n = AppLocalizations.of(context);
  final auth = ProviderScope.containerOf(context, listen: false).read(authProvider.notifier);
  try {
    final result = await apiCall();
    if (!result.ok) {
      setError(l10n != null
          ? friendlyError(l10n, result)
          : result.error ?? 'Unknown error');
      if (context.mounted && isIpRateLimited(result.error)) {
        showIpRateLimitBanner(context);
      }
      await auth.reportFailure(result);
      return;
    }
    final decoded = tryJsonDecode(result.data);
    if (decoded == null) {
      setError(l10n?.invalidResponse ?? 'Invalid response from server');
      return;
    }
    await onData?.call(decoded);
  } catch (e) {
    final err = e.toString();
    debugPrint('API call failed: $err');
    setError(l10n?.errorUnexpectedResponse ?? err);
    if (context.mounted && isIpRateLimited(err)) {
      showIpRateLimitBanner(context);
    }
  } finally {
    setLoading(false);
  }
}
