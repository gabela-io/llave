import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/app_localizations.dart';
import '../src/api_helpers.dart';
import '../src/auth_provider.dart';
import '../src/biometric_service.dart';
import '../src/errors.dart';
import '../src/ip_rate_limit.dart';
import '../src/llave_bridge.dart';

/// Reveals the pending Cl@ve PIN (requested on the AEAT website) with a
/// countdown, like the official app's "PIN" card.
class PinCard extends ConsumerStatefulWidget {
  const PinCard({super.key, this.confirmPresence});

  /// Asked before revealing the PIN. Defaults to a biometric check when
  /// biometrics are enabled; otherwise the unlocked app is enough.
  final Future<bool> Function()? confirmPresence;

  @override
  ConsumerState<PinCard> createState() => _PinCardState();
}

class _PinCardState extends ConsumerState<PinCard> {
  String? _pin;
  int _remaining = 0;
  bool _loading = false;
  String? _message;
  Timer? _timer;

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  Future<bool> _defaultConfirm(String reason) async {
    if (!await BiometricService.isEnabled()) return true;
    return await BiometricService.authenticate(localizedReason: reason) != null;
  }

  Future<void> _reveal() async {
    final l10n = AppLocalizations.of(context)!;
    final confirm = widget.confirmPresence ?? () => _defaultConfirm(l10n.pinBiometricReason);
    setState(() {
      _loading = true;
      _message = null;
    });
    try {
      if (!await confirm()) return;
      final result = await ref.read(llaveBridgeProvider).requestPin();
      if (!mounted) return;
      final data = result.ok ? tryJsonDecode(result.data) : null;
      final pin = data is Map<String, dynamic> ? data['pin'] as String? : null;
      if (pin == null) {
        setState(() => _message =
            result.ok ? l10n.errorUnexpectedResponse : friendlyError(l10n, result));
        if (!result.ok) {
          if (isIpRateLimited(result.error)) showIpRateLimitBanner(context);
          await ref.read(authProvider.notifier).reportFailure(result);
        }
        return;
      }
      _show(pin, int.tryParse(data['time_to_live_seconds'] as String? ?? ''));
    } catch (e) {
      if (mounted) setState(() => _message = l10n.errorUnexpectedResponse);
    } finally {
      if (mounted) setState(() => _loading = false);
    }
  }

  /// Show [pin]; with a validity, count down and hide it when it expires.
  /// Without one it stays until the user hides it.
  void _show(String pin, int? seconds) {
    _timer?.cancel();
    setState(() {
      _pin = pin;
      _remaining = seconds ?? 0;
    });
    if (seconds == null || seconds <= 0) return;
    _timer = Timer.periodic(const Duration(seconds: 1), (_) {
      if (_remaining <= 1) {
        _hide();
      } else {
        setState(() => _remaining--);
      }
    });
  }

  void _hide() {
    _timer?.cancel();
    setState(() {
      _pin = null;
      _remaining = 0;
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);
    final pin = _pin;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(l10n.pinCardTitle, style: theme.textTheme.titleMedium),
            const SizedBox(height: 8),
            if (pin == null) ...[
              Text(
                _message ?? l10n.pinCardHint,
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: _message == null
                      ? theme.colorScheme.onSurfaceVariant
                      : theme.colorScheme.error,
                ),
              ),
              const SizedBox(height: 12),
              FilledButton.tonalIcon(
                onPressed: _loading ? null : _reveal,
                icon: _loading
                    ? const SizedBox.square(
                        dimension: 18,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Icon(Icons.pin_outlined),
                label: Text(l10n.pinReveal),
              ),
            ] else ...[
              SelectableText(
                pin,
                style: theme.textTheme.displaySmall?.copyWith(
                  fontFeatures: const [FontFeature.tabularFigures()],
                  letterSpacing: 4,
                ),
              ),
              const SizedBox(height: 4),
              Row(
                children: [
                  Expanded(
                    child: _remaining > 0 ? Text(l10n.pinValidFor(_remaining)) : const SizedBox(),
                  ),
                  TextButton(onPressed: _hide, child: Text(l10n.pinHide)),
                ],
              ),
            ],
          ],
        ),
      ),
    );
  }
}
