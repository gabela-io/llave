import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../src/api_helpers.dart';
import '../src/llave_bridge.dart';
import '../src/rust/api/api.dart' as native_ffi;
import '../widgets/data_screen_scaffold.dart';

class PendingScreen extends StatefulWidget {
  const PendingScreen({super.key});

  @override
  State<PendingScreen> createState() => _PendingScreenState();
}

class _PendingScreenState extends State<PendingScreen> {
  bool _loading = false;
  List<Map<String, dynamic>> _requests = [];
  String? _error;

  @override
  void initState() {
    super.initState();
    _refresh();
  }

  Future<void> _refresh() async {
    setState(() {
      _loading = true;
      _error = null;
    });

    await handleApiCall(
      apiCall: () async {
        final r = await native_ffi.getPendingRequests();
        return LlaveApiResult(ok: r.ok, data: r.data, error: r.error);
      },
      setLoading: (v) {
        if (mounted) setState(() => _loading = v);
      },
      setError: (e) {
        if (mounted) setState(() => _error = e);
      },
      context: context,
      onData: (decoded) {
        final data = decoded['data'];
        final List<Map<String, dynamic>> items = [];
        if (data is Map<String, dynamic>) {
          final peticion = data['peticion'];
          if (peticion is Map<String, dynamic> && peticion.isNotEmpty) {
            items.add(Map<String, dynamic>.from(peticion));
          }
        } else if (data is List) {
          for (final item in data) {
            if (item is Map<String, dynamic>) items.add(item);
          }
        }
        setState(() => _requests = items);
      },
    );
  }

  Future<void> _confirm(String token, String idpCode) async {
    final l10n = AppLocalizations.of(context)!;
    try {
      final result =
          await native_ffi.confirmRequest(token: token, idpCode: idpCode);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
              content: Text(
                  result.ok ? l10n.requestConfirmed : (result.error ?? l10n.failed))),
        );
        _refresh();
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(l10n.errorWithMessage(e.toString()))),
        );
      }
    }
  }

  Future<void> _reject(String token, String idpCode) async {
    final l10n = AppLocalizations.of(context)!;
    try {
      final result =
          await native_ffi.rejectRequest(token: token, idpCode: idpCode);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
              content: Text(
                  result.ok ? l10n.requestRejected : (result.error ?? l10n.failed))),
        );
        _refresh();
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(l10n.errorWithMessage(e.toString()))),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.pendingTitle),
        actions: [
          IconButton(
            onPressed: _loading ? null : _refresh,
            icon: const Icon(Icons.refresh),
            tooltip: l10n.refresh,
          ),
        ],
      ),
      body: DataScreenScaffold(
        loading: _loading,
        error: _error,
        isEmpty: _requests.isEmpty,
        onRetry: _refresh,
        emptyIcon: Icons.inbox_outlined,
        emptyTitle: l10n.pendingEmptyTitle,
        emptySubtitle: l10n.pendingEmptySubtitle,
        child: RefreshIndicator(
          onRefresh: _refresh,
          child: ListView.builder(
            padding: const EdgeInsets.all(16),
            itemCount: _requests.length,
            itemBuilder: (context, index) {
              final req = _requests[index];
              final token = req['tokenClaveMovil']?.toString() ?? '';
              final idpCode = req['codigoIdP']?.toString() ?? '';
              final organismo = req['nombreProveedor']?.toString() ??
                  req['organismo']?.toString() ??
                  l10n.unknownProvider;

              return Card(
                margin: const EdgeInsets.only(bottom: 12),
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Icon(Icons.notifications_active,
                              color: theme.colorScheme.primary),
                          const SizedBox(width: 12),
                          Expanded(
                            child: Text(
                              organismo,
                              style: theme.textTheme.titleSmall,
                            ),
                          ),
                        ],
                      ),
                      if (idpCode.isNotEmpty) ...[
                        const SizedBox(height: 8),
                        Text(
                          l10n.providerCode(idpCode),
                          style: theme.textTheme.bodySmall?.copyWith(
                            color: theme.colorScheme.onSurfaceVariant,
                          ),
                        ),
                      ],
                      const SizedBox(height: 16),
                      Row(
                        mainAxisAlignment: MainAxisAlignment.end,
                        children: [
                          OutlinedButton.icon(
                            onPressed: () => _showConfirmDialog(
                              context,
                              reject: true,
                              onConfirm: () => _reject(token, idpCode),
                            ),
                            icon: const Icon(Icons.close, size: 18),
                            label: Text(l10n.reject),
                            style: OutlinedButton.styleFrom(
                              foregroundColor: theme.colorScheme.error,
                            ),
                          ),
                          const SizedBox(width: 12),
                          FilledButton.icon(
                            onPressed: () => _showConfirmDialog(
                              context,
                              reject: false,
                              onConfirm: () => _confirm(token, idpCode),
                            ),
                            icon: const Icon(Icons.check, size: 18),
                            label: Text(l10n.confirm),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
              );
            },
          ),
        ),
      ),
    );
  }

  void _showConfirmDialog(
    BuildContext context, {
    required bool reject,
    required VoidCallback onConfirm,
  }) {
    final l10n = AppLocalizations.of(context)!;
    final action = reject ? l10n.reject : l10n.confirm;
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(l10n.actionRequestTitle(action)),
        content: Text(l10n.actionRequestMessage(action.toLowerCase())),
        actions: [
          TextButton(
              onPressed: () => Navigator.pop(ctx),
              child: Text(l10n.cancel)),
          FilledButton(
            onPressed: () {
              Navigator.pop(ctx);
              onConfirm();
            },
            child: Text(action),
          ),
        ],
      ),
    );
  }
}
