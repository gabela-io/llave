import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../l10n/app_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:smart_auth/smart_auth.dart';

import '../src/api_helpers.dart';
import '../src/auth_provider.dart';
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
  final _smsController = TextEditingController();
  bool _loading = false;
  String? _error;
  bool _activated = false;

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

  /// Open a date picker and fill the date field in DD/MM/YYYY format.
  Future<void> _pickDate() async {
    final l10n = AppLocalizations.of(context)!;
    // Try to parse an existing value as initial date.
    DateTime initial = DateTime.now();
    final text = _fechaController.text.trim();
    if (RegExp(r'^\d{2}/\d{2}/\d{4}$').hasMatch(text)) {
      final parts = text.split('/');
      final parsed = DateTime.tryParse('${parts[2]}-${parts[1]}-${parts[0]}');
      if (parsed != null) initial = parsed;
    }

    final picked = await showDatePicker(
      context: context,
      initialDate: initial,
      firstDate: DateTime(2000),
      lastDate: DateTime(2040),
      helpText: l10n.fechaValidezLabel,
    );
    if (picked != null) {
      _fechaController.text =
          '${picked.day.toString().padLeft(2, '0')}/'
          '${picked.month.toString().padLeft(2, '0')}/'
          '${picked.year}';
    }
  }

  /// Phase 1: DNI auth → request SMS.
  Future<void> _authenticate() async {
    if (!_formKey.currentState!.validate()) return;

    setState(() {
      _loading = true;
      _error = null;
    });

    final authNotifier = ref.read(authProvider.notifier);

    await handleApiCall(
      apiCall: () => authNotifier.dniAuthenticate(
        _nifController.text.trim(),
        _fechaController.text.trim(),
        '',
      ),
      setLoading: (v) {
        if (mounted) setState(() => _loading = v);
      },
      setError: (e) {
        if (mounted) setState(() => _error = e);
      },
      context: context,
      onData: (decoded) {
        final data = decoded as Map<String, dynamic>;
        setState(() {
          _maskedPhone = data['movil'] as String?;
          _cookiesJson = data['cookies_json'] as String?;
          _timestampAltaSms = data['timestamp_alta_sms'] as String?;
          _tokenClaveMovilSms = data['token_clave_movil_sms'] as String?;
        });
        _listenForSmsCode();
      },
    );
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
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go(AppRoutes.activate),
        ),
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
            validator: (v) {
              if (v == null || v.trim().isEmpty) return l10n.nifRequired;
              if (v.trim().length < 8) return l10n.nifMinLength;
              return null;
            },
          ),
          const SizedBox(height: 16),

          // Fecha (expiry date)
          TextFormField(
            controller: _fechaController,
            decoration: InputDecoration(
              labelText: l10n.fechaValidezLabel,
              hintText: l10n.ddMmYyyy,
              border: const OutlineInputBorder(),
              prefixIcon: const Icon(Icons.calendar_today),
              helperText: l10n.fechaHelperText,
              suffixIcon: (Platform.isAndroid || Platform.isIOS) && !_loading
                  ? IconButton(
                      icon: const Icon(Icons.edit_calendar),
                      onPressed: _pickDate,
                    )
                  : null,
            ),
            keyboardType: TextInputType.datetime,
            readOnly: Platform.isAndroid || Platform.isIOS,
            onTap: (Platform.isAndroid || Platform.isIOS) && !_loading
                ? _pickDate
                : null,
            enabled: !_loading,
            validator: (v) {
              if (v == null || v.trim().isEmpty) return l10n.dateRequired;
              final trimmed = v.trim();
              if (!RegExp(r'^\d{2}/\d{2}/\d{4}$').hasMatch(trimmed)) {
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
