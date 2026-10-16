import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../l10n/app_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:smart_auth/smart_auth.dart';

import '../src/api_helpers.dart';
import '../src/auth_provider.dart';
import '../src/errors.dart';
import '../src/ip_rate_limit.dart';
import '../src/llave_bridge.dart';
import '../src/routes.dart';

class DniAuthScreen extends ConsumerStatefulWidget {
  const DniAuthScreen({super.key});

  @override
  ConsumerState<DniAuthScreen> createState() => _DniAuthScreenState();
}

class _DniAuthScreenState extends ConsumerState<DniAuthScreen> {
  final _formKey = GlobalKey<FormState>();
  final _nifController = TextEditingController();
  final _fechaController = TextEditingController();
  final _soporteController = TextEditingController();
  final _smsController = TextEditingController();
  bool _loading = false;
  String? _error;
  bool _activated = false;

  // Document field for the typed NIF (`validity_date`, `birth_date`,
  // `support_number`), and the server's date-of-birth challenge, if any.
  String? _input;
  String? _challenge;
  String? _challengeMessage;

  // Phase 1 result — stored for phase 2.
  String? _maskedPhone;
  String? _cookiesJson;
  String? _timestampAltaSms;
  String? _tokenClaveMovilSms;

  @override
  void dispose() {
    if (Platform.isAndroid) {
      SmartAuth.instance.removeUserConsentApiListener();
    }
    _nifController.dispose();
    _fechaController.dispose();
    _soporteController.dispose();
    _smsController.dispose();
    super.dispose();
  }

  /// Start listening for incoming SMS on Android using SMS User Consent API.
  /// Shows a system dialog when SMS arrives, asking the user to share it.
  void _listenForSmsCode() {
    if (!Platform.isAndroid) return;

    SmartAuth.instance.getSmsWithUserConsentApi().then((res) {
      if (!mounted) return;
      if (res.hasData) {
        final code = res.requireData.code;
        if (code != null && code.isNotEmpty) {
          setState(() {
            _smsController.text = code;
          });
        }
      }
    });
  }

  /// The document field to ask for: the server's date-of-birth challenge
  /// wins, otherwise it follows the document type (DNI by default).
  _DniField get _field => _challenge != null
      ? _DniField.birthDate
      : _DniField.fromWire(_input) ?? _DniField.validityDate;

  void _onNifChanged(String value) {
    String? input;
    try {
      input = ref.read(llaveBridgeProvider).dniInput(value);
    } catch (_) {
      // Not a valid NIF yet.
    }
    setState(() {
      _input = input;
      _challenge = null;
    });
  }

  /// Open a date picker and fill the date field in DD-MM-YYYY format.
  Future<void> _pickDate() async {
    final birthDate = _field == _DniField.birthDate;
    final now = DateTime.now();
    DateTime initial = birthDate ? DateTime(now.year - 40) : now;
    final match = RegExp(r'^(\d{2})-(\d{2})-(\d{4})$').firstMatch(_fechaController.text.trim());
    if (match != null) {
      initial = DateTime.tryParse('${match[3]}-${match[2]}-${match[1]}') ?? initial;
    }
    final first = birthDate ? DateTime(1900) : DateTime(2000);
    final last = birthDate ? now : DateTime(now.year + 20);
    if (initial.isBefore(first) || initial.isAfter(last)) initial = last;

    final picked = await showDatePicker(
      context: context,
      initialDate: initial,
      firstDate: first,
      lastDate: last,
      helpText: _field.label(AppLocalizations.of(context)!),
    );
    if (picked != null) {
      _fechaController.text = '${picked.day.toString().padLeft(2, '0')}-'
          '${picked.month.toString().padLeft(2, '0')}-'
          '${picked.year}';
    }
  }

  /// Phase 1: DNI auth → request SMS. When the server asks for the date of
  /// birth, the date field is relabelled and the user submits again.
  Future<void> _authenticate() async {
    if (!_formKey.currentState!.validate()) return;

    final l10n = AppLocalizations.of(context)!;
    final support = _field == _DniField.supportNumber;
    final fecha = support ? '' : _fechaController.text.trim();
    final soporte = support ? _soporteController.text.trim() : '';

    setState(() {
      _loading = true;
      _error = null;
    });

    try {
      final result = await ref.read(authProvider.notifier).dniAuthenticate(
            _nifController.text.trim(),
            fecha,
            soporte,
            challenge: _challenge ?? '',
          );
      if (!mounted) return;

      if (result.ok) {
        final data = tryJsonDecode(result.data);
        if (data is! Map<String, dynamic>) {
          setState(() => _error = l10n.invalidResponse);
          return;
        }
        setState(() {
          _maskedPhone = data['movil'] as String?;
          _cookiesJson = data['cookies_json'] as String?;
          _timestampAltaSms = data['timestamp_alta_sms'] as String?;
          _tokenClaveMovilSms = data['token_clave_movil_sms'] as String?;
        });
        _listenForSmsCode();
        return;
      }

      final data = tryJsonDecode(result.data);
      final challenge = data is Map<String, dynamic> ? data['challenge'] as String? : null;
      if (challenge != null) {
        setState(() {
          _challenge = challenge;
          _challengeMessage = result.serverMessage;
          _fechaController.clear();
        });
        return;
      }
      _showError(friendlyError(l10n, result), raw: result.error);
    } catch (e) {
      if (mounted) _showError(e.toString());
    } finally {
      if (mounted) setState(() => _loading = false);
    }
  }

  void _showError(String message, {String? raw}) {
    setState(() => _error = message);
    if (isIpRateLimited(raw ?? message)) showIpRateLimitBanner(context);
  }

  /// Phase 2: Validate SMS PIN → activate device.
  Future<void> _submitSmsPin() async {
    final pin = _smsController.text.trim();
    if (pin.isEmpty) return;

    setState(() {
      _loading = true;
      _error = null;
    });

    final authNotifier = ref.read(authProvider.notifier);

    await handleApiCall(
      apiCall: () => authNotifier.dniCompleteActivation(
        _nifController.text.trim(),
        _cookiesJson ?? '',
        _timestampAltaSms ?? '',
        _tokenClaveMovilSms ?? '',
        pin,
      ),
      setLoading: (v) {
        if (mounted) setState(() => _loading = v);
      },
      setError: (e) {
        if (mounted) setState(() => _error = e);
      },
      context: context,
      onData: (_) {
        TextInput.finishAutofillContext();
        setState(() => _activated = true);
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.dniAuthTitle),
      ),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: _activated
              ? _buildSuccessView(theme, l10n)
              : _maskedPhone != null
                  ? _buildSmsView(theme, l10n)
                  : _buildForm(theme, l10n),
        ),
      ),
    );
  }

  Widget _buildSuccessView(ThemeData theme, AppLocalizations l10n) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Card(
          color: theme.colorScheme.primaryContainer,
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Row(
              children: [
                Icon(Icons.check_circle, color: theme.colorScheme.primary),
                const SizedBox(width: 12),
                Expanded(
                  child: Text(
                    l10n.deviceActivated,
                    style: TextStyle(color: theme.colorScheme.onPrimaryContainer),
                  ),
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 16),
        FilledButton.icon(
          onPressed: () => context.go(AppRoutes.home),
          icon: const Icon(Icons.home),
          label: Text(l10n.goToHome),
        ),
      ],
    );
  }

  Widget _buildSmsView(ThemeData theme, AppLocalizations l10n) {
    return AutofillGroup(
      child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Icon(
          Icons.sms_outlined,
          size: 64,
          color: theme.colorScheme.primary,
        ),
        const SizedBox(height: 16),
        Text(
          l10n.smsVerification,
          style: theme.textTheme.headlineMedium?.copyWith(
            fontWeight: FontWeight.bold,
          ),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 8),
        Text(
          l10n.smsSentTo,
          style: theme.textTheme.bodyMedium?.copyWith(
            color: theme.colorScheme.onSurfaceVariant,
          ),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 8),
        Text(
          _maskedPhone ?? '***',
          style: theme.textTheme.titleLarge?.copyWith(
            fontWeight: FontWeight.bold,
            letterSpacing: 2,
          ),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 24),
        TextFormField(
          controller: _smsController,
          decoration: InputDecoration(
            labelText: l10n.smsCodeLabel,
            hintText: l10n.smsCodeHint,
            border: const OutlineInputBorder(),
            prefixIcon: const Icon(Icons.pin),
          ),
          keyboardType: TextInputType.number,
          autofillHints: const [AutofillHints.oneTimeCode],
          textAlign: TextAlign.center,
          style: const TextStyle(fontSize: 24, letterSpacing: 8),
          enabled: !_loading,
        ),

        if (_error != null) ...[
          const SizedBox(height: 16),
          Card(
            color: theme.colorScheme.errorContainer,
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: Row(
                children: [
                  Icon(Icons.error_outline,
                      color: theme.colorScheme.error, size: 20),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      _error!,
                      style:
                          TextStyle(color: theme.colorScheme.onErrorContainer),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],

        const SizedBox(height: 24),
        FilledButton.icon(
          onPressed: _loading ? null : _submitSmsPin,
          icon: _loading
              ? const SizedBox(
                  width: 18,
                  height: 18,
                  child: CircularProgressIndicator(
                      strokeWidth: 2, color: Colors.white),
                )
              : const Icon(Icons.check),
          label: Text(_loading ? l10n.verifying : l10n.verifyActivate),
          style: FilledButton.styleFrom(
            padding: const EdgeInsets.symmetric(vertical: 16),
          ),
        ),
      ],
    ),
    );
  }

  Widget _buildForm(ThemeData theme, AppLocalizations l10n) {
    return Form(
      key: _formKey,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Icon(
            Icons.badge_outlined,
            size: 64,
            color: theme.colorScheme.primary,
          ),
          const SizedBox(height: 16),
          Text(
            l10n.dniLogin,
            style: theme.textTheme.headlineMedium?.copyWith(
              fontWeight: FontWeight.bold,
            ),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 8),
          Text(
            l10n.dniLoginDescription,
            style: theme.textTheme.bodyMedium?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
            ),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 32),

          // NIF field
          TextFormField(
            controller: _nifController,
            decoration: InputDecoration(
              labelText: l10n.nifDniNie,
              hintText: l10n.nifHint,
              border: const OutlineInputBorder(),
              prefixIcon: const Icon(Icons.badge),
            ),
            textCapitalization: TextCapitalization.characters,
            enabled: !_loading,
            onChanged: _onNifChanged,
            validator: (v) {
              if (v == null || v.trim().isEmpty) return l10n.nifRequired;
              if (v.trim().length < 8) return l10n.nifMinLength;
              return null;
            },
          ),
          const SizedBox(height: 16),

          if (_challenge != null) ...[
            Card(
              color: theme.colorScheme.secondaryContainer,
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Row(
                  children: [
                    Icon(Icons.info_outline,
                        color: theme.colorScheme.onSecondaryContainer, size: 20),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        (_challengeMessage?.isNotEmpty ?? false)
                            ? _challengeMessage!
                            : l10n.birthDateChallenge,
                        style: TextStyle(color: theme.colorScheme.onSecondaryContainer),
                      ),
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),
          ],

          if (_field == _DniField.supportNumber)
            TextFormField(
              controller: _soporteController,
              decoration: InputDecoration(
                labelText: l10n.supportNumberLabel,
                hintText: 'E12345678',
                border: const OutlineInputBorder(),
                prefixIcon: const Icon(Icons.numbers),
                helperText: l10n.supportNumberHelperText,
              ),
              textCapitalization: TextCapitalization.characters,
              // The app accepts letters and digits only.
              inputFormatters: [FilteringTextInputFormatter.allow(RegExp('[A-Za-z0-9]'))],
              enabled: !_loading,
              validator: (v) =>
                  (v == null || v.trim().isEmpty) ? l10n.supportNumberRequired : null,
            )
          else
            TextFormField(
              key: ValueKey(_field),
              controller: _fechaController,
              decoration: InputDecoration(
                labelText: _field.label(l10n),
                hintText: l10n.ddMmYyyy,
                border: const OutlineInputBorder(),
                prefixIcon: const Icon(Icons.calendar_today),
                helperText: _field == _DniField.birthDate
                    ? l10n.birthDateHelperText
                    : l10n.fechaHelperText,
                suffixIcon: (Platform.isAndroid || Platform.isIOS) && !_loading
                    ? IconButton(
                        icon: const Icon(Icons.edit_calendar),
                        onPressed: _pickDate,
                      )
                    : null,
              ),
              keyboardType: TextInputType.datetime,
              readOnly: Platform.isAndroid || Platform.isIOS,
              onTap: (Platform.isAndroid || Platform.isIOS) && !_loading ? _pickDate : null,
              enabled: !_loading,
              validator: (v) {
                if (v == null || v.trim().isEmpty) return l10n.dateRequired;
                if (!RegExp(r'^\d{2}-\d{2}-\d{4}$').hasMatch(v.trim())) {
                  return l10n.useDdMmYyyy;
                }
                return null;
              },
            ),

          // Error message
          if (_error != null) ...[
            const SizedBox(height: 16),
            Card(
              color: theme.colorScheme.errorContainer,
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Row(
                  children: [
                    Icon(Icons.error_outline,
                        color: theme.colorScheme.error, size: 20),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        _error!,
                        style:
                            TextStyle(color: theme.colorScheme.onErrorContainer),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ],

          const SizedBox(height: 24),

          // Submit button
          FilledButton.icon(
            onPressed: _loading ? null : _authenticate,
            icon: _loading
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(
                        strokeWidth: 2, color: Colors.white),
                  )
                : const Icon(Icons.login),
            label: Text(_loading ? l10n.authenticating : l10n.authenticate),
            style: FilledButton.styleFrom(
              padding: const EdgeInsets.symmetric(vertical: 16),
            ),
          ),
        ],
      ),
    );
  }
}

/// Document field the DNI/NIE authentication asks for.
enum _DniField {
  validityDate,
  birthDate,
  supportNumber;

  /// Parses the core's `dni_input` value.
  static _DniField? fromWire(String? value) => switch (value) {
        'validity_date' => validityDate,
        'birth_date' => birthDate,
        'support_number' => supportNumber,
        _ => null,
      };

  String label(AppLocalizations l10n) => switch (this) {
        validityDate => l10n.fechaValidezLabel,
        birthDate => l10n.birthDateLabel,
        supportNumber => l10n.supportNumberLabel,
      };
}
