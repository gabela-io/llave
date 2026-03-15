import 'dart:io';

import 'package:flutter/material.dart';
import '../l10n/app_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import '../src/api_helpers.dart';
import '../src/llave_bridge.dart';
import '../src/routes.dart';

/// Whether the current platform supports camera-based QR scanning.
bool get _hasCameraSupport => Platform.isAndroid || Platform.isIOS;

class QrAuthScreen extends ConsumerStatefulWidget {
  const QrAuthScreen({super.key});

  @override
  ConsumerState<QrAuthScreen> createState() => _QrAuthScreenState();
}

class _QrAuthScreenState extends ConsumerState<QrAuthScreen> {
  final _controller = TextEditingController();
  bool _loading = false;
  String? _error;
  bool _success = false;
  bool _useManualInput = false;

  @override
  void initState() {
    super.initState();
    _useManualInput = !_hasCameraSupport;
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _authenticate(String value) async {
    if (value.trim().isEmpty) return;

    setState(() {
      _loading = true;
      _error = null;
      _success = false;
    });

    final bridge = ref.read(llaveBridgeProvider);

    await handleApiCall(
      apiCall: () => bridge.qrAuthenticate(value.trim()),
      setLoading: (v) {
        if (mounted) setState(() => _loading = v);
      },
      setError: (e) {
        if (mounted) setState(() => _error = e);
      },
      context: context,
      onData: (decoded) async {
        final l10n = AppLocalizations.of(context)!;
        final peticion = decoded is Map<String, dynamic>
            ? (decoded['peticion'] as Map<String, dynamic>?)
            : null;

        if (peticion == null) {
          setState(() => _error = l10n.invalidPetitionData);
          return;
        }

        final token = peticion['tokenClaveMovil'] as String? ?? '';
        final idpCode = peticion['codigoIdP'] as String? ?? '';
        final organismo = peticion['organismo'] as String? ?? '';
        final tipo = peticion['tipo'] as String? ?? '';

        setState(() => _loading = false);

        if (!mounted) return;
        final confirmed = await _showConfirmDialog(organismo, tipo);
        if (!mounted) return;

        if (confirmed == true) {
          await _confirmRequest(bridge, token, idpCode);
        } else {
          await _rejectRequest(bridge, token, idpCode);
        }
      },
    );
  }

  Future<bool?> _showConfirmDialog(String organismo, String tipo) {
    final l10n = AppLocalizations.of(context)!;
    return showDialog<bool>(
      context: context,
      barrierDismissible: false,
      builder: (ctx) => AlertDialog(
        icon: const Icon(Icons.verified_user, size: 40),
        title: Text(l10n.confirmAuthTitle),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(l10n.authRequestReceived),
            const SizedBox(height: 16),
            if (organismo.isNotEmpty) ...[
              Text(l10n.organizationLabel, style: Theme.of(ctx).textTheme.labelSmall),
              Text(organismo, style: Theme.of(ctx).textTheme.bodyLarge),
              const SizedBox(height: 8),
            ],
            if (tipo.isNotEmpty) ...[
              Text(l10n.typeLabel, style: Theme.of(ctx).textTheme.labelSmall),
              Text(tipo, style: Theme.of(ctx).textTheme.bodyLarge),
            ],
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: Text(l10n.reject),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: Text(l10n.confirm),
          ),
        ],
      ),
    );
  }

  Future<void> _confirmRequest(
      LlaveBridge bridge, String token, String idpCode) async {
    setState(() {
      _loading = true;
      _error = null;
    });

    await handleApiCall(
      apiCall: () => bridge.confirmRequest(token, idpCode),
      setLoading: (v) {
        if (mounted) setState(() => _loading = v);
      },
      setError: (e) {
        if (mounted) setState(() => _error = e);
      },
      context: context,
      onData: (_) {
        setState(() => _success = true);
      },
    );
  }

  Future<void> _rejectRequest(
      LlaveBridge bridge, String token, String idpCode) async {
    setState(() {
      _loading = true;
      _error = null;
    });

    try {
      await bridge.rejectRequest(token, idpCode);
    } catch (_) {
      // Rejection errors are not critical.
    }

    if (!mounted) return;
    setState(() {
      _loading = false;
      _error = null;
      _success = false;
    });
  }

  void _onQrDetected(BarcodeCapture capture) {
    if (_loading || _success) return;
    final barcode = capture.barcodes.firstOrNull;
    if (barcode?.rawValue == null) return;
    _authenticate(barcode!.rawValue!);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go(AppRoutes.home),
        ),
        title: Text(l10n.qrAuthTitle),
        actions: [
          if (_hasCameraSupport)
            IconButton(
              icon: Icon(_useManualInput ? Icons.camera_alt : Icons.keyboard),
              tooltip: _useManualInput ? l10n.useCamera : l10n.enterManually,
              onPressed: () =>
                  setState(() => _useManualInput = !_useManualInput),
            ),
        ],
      ),
      body: _success
          ? _buildSuccessView(theme, l10n)
          : _useManualInput
              ? _buildManualInput(theme, l10n)
              : _buildCameraScanner(theme, l10n),
    );
  }

  Widget _buildSuccessView(ThemeData theme, AppLocalizations l10n) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.check_circle, size: 64, color: Colors.green),
            const SizedBox(height: 16),
            Text(
              l10n.qrSuccess,
              style: theme.textTheme.titleMedium,
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 24),
            OutlinedButton.icon(
              onPressed: () => setState(() {
                _success = false;
                _controller.clear();
              }),
              icon: const Icon(Icons.qr_code),
              label: Text(l10n.scanAnother),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildCameraScanner(ThemeData theme, AppLocalizations l10n) {
    return Column(
      children: [
        Expanded(
          child: Stack(
            children: [
              MobileScanner(onDetect: _onQrDetected),
              // Overlay with scanning guide
              Center(
                child: Container(
                  width: 250,
                  height: 250,
                  decoration: BoxDecoration(
                    border: Border.all(
                      color: theme.colorScheme.primary.withValues(alpha: 0.7),
                      width: 3,
                    ),
                    borderRadius: BorderRadius.circular(16),
                  ),
                ),
              ),
              // Loading overlay
              if (_loading)
                Container(
                  color: Colors.black54,
                  child: const Center(
                    child: CircularProgressIndicator(color: Colors.white),
                  ),
                ),
            ],
          ),
        ),
        // Status bar at bottom
        Container(
          width: double.infinity,
          padding: const EdgeInsets.all(16),
          child: Column(
            children: [
              if (_error != null) ...[
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
                            style: TextStyle(
                                color: theme.colorScheme.onErrorContainer),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ] else
                Text(
                  _loading
                      ? l10n.authenticating
                      : l10n.pointCamera,
                  style: theme.textTheme.bodyMedium?.copyWith(
                    color: theme.colorScheme.onSurfaceVariant,
                  ),
                  textAlign: TextAlign.center,
                ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildManualInput(ThemeData theme, AppLocalizations l10n) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.qr_code_2, size: 64, color: theme.colorScheme.primary),
            const SizedBox(height: 16),
            Text(
              l10n.qrCodeAuthentication,
              style: theme.textTheme.titleMedium,
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 8),
            Text(
              l10n.pasteQrHint,
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 32),

            if (_error != null) ...[
              Card(
                color: theme.colorScheme.errorContainer,
                child: Padding(
                  padding: const EdgeInsets.all(12),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(Icons.error_outline,
                          color: theme.colorScheme.error, size: 20),
                      const SizedBox(width: 8),
                      Flexible(
                        child: Text(
                          _error!,
                          style: TextStyle(
                              color: theme.colorScheme.onErrorContainer),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(height: 16),
            ],

            // QR value input
            ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 400),
              child: TextField(
                controller: _controller,
                decoration: InputDecoration(
                  labelText: l10n.qrCodeValueLabel,
                  hintText: l10n.pasteQrContent,
                  border: const OutlineInputBorder(),
                  prefixIcon: const Icon(Icons.qr_code),
                ),
                maxLines: 3,
                minLines: 1,
                enabled: !_loading,
                onSubmitted: (_) => _authenticate(_controller.text),
              ),
            ),
            const SizedBox(height: 16),

            FilledButton.icon(
              onPressed:
                  _loading ? null : () => _authenticate(_controller.text),
              icon: _loading
                  ? const SizedBox(
                      width: 18,
                      height: 18,
                      child: CircularProgressIndicator(
                          strokeWidth: 2, color: Colors.white),
                    )
                  : const Icon(Icons.login),
              label: Text(_loading ? l10n.authenticating : l10n.authenticate),
            ),
          ],
        ),
      ),
    );
  }
}
