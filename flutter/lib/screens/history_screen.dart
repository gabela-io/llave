import 'package:flutter/material.dart';
import '../l10n/app_localizations.dart';

import '../src/api_helpers.dart';
import '../src/llave_bridge.dart';
import '../src/rust/api/api.dart' as native_ffi;
import '../widgets/data_screen_scaffold.dart';

class HistoryScreen extends StatefulWidget {
  const HistoryScreen({super.key});

  @override
  State<HistoryScreen> createState() => _HistoryScreenState();
}

class _HistoryScreenState extends State<HistoryScreen> {
  bool _loading = false;
  List<Map<String, dynamic>> _operations = [];
  String? _error;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
    });

    await handleApiCall(
      apiCall: () async {
        final r = await native_ffi.getHistory();
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
        final List<Map<String, dynamic>> items = [];
        if (decoded is List) {
          for (final item in decoded) {
            if (item is Map<String, dynamic>) items.add(item);
          }
        }
        setState(() => _operations = items);
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.historyTitle),
        actions: [
          IconButton(
            onPressed: _loading ? null : _load,
            icon: const Icon(Icons.refresh),
            tooltip: l10n.refresh,
          ),
        ],
      ),
      body: DataScreenScaffold(
        loading: _loading,
        error: _error,
        isEmpty: _operations.isEmpty,
        onRetry: _load,
        emptyIcon: Icons.history,
        emptyTitle: l10n.historyEmptyTitle,
        child: RefreshIndicator(
          onRefresh: _load,
          child: ListView.builder(
            padding: const EdgeInsets.all(16),
            itemCount: _operations.length,
            itemBuilder: (context, index) =>
                _OperationCard(operation: _operations[index]),
          ),
        ),
      ),
    );
  }
}

class _OperationCard extends StatelessWidget {
  final Map<String, dynamic> operation;
  const _OperationCard({required this.operation});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);
    final tipo = operation['tipo']?.toString() ?? '';
    final resultado = operation['resultado']?.toString() ?? '';
    final organismo = operation['organismo']?.toString() ?? '';
    final fecha = operation['fecha']?.toString() ?? '';

    // "01" = confirmed/accepted, "10" = rejected.
    final isAccepted = resultado == '01';
    final resultLabel = isAccepted ? l10n.accepted : l10n.rejected;
    final resultColor = isAccepted ? Colors.green : theme.colorScheme.error;

    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: ListTile(
        leading: Icon(
          _iconForType(tipo),
          color: resultColor,
        ),
        title: Text(organismo.isNotEmpty ? organismo : _labelForType(l10n, tipo)),
        subtitle: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(_labelForType(l10n, tipo), style: theme.textTheme.bodySmall),
            if (fecha.isNotEmpty)
              Text(
                fecha,
                style: theme.textTheme.bodySmall?.copyWith(
                  color: theme.colorScheme.onSurfaceVariant,
                ),
              ),
          ],
        ),
        trailing: Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          decoration: BoxDecoration(
            color: resultColor.withValues(alpha: 0.1),
            borderRadius: BorderRadius.circular(8),
          ),
          child: Text(
            resultLabel,
            style: theme.textTheme.labelSmall?.copyWith(
              color: resultColor,
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
      ),
    );
  }

  /// Decode the 3-digit binary type bitmask:
  /// bit 0 (rightmost) = DNI/NIE, bit 1 = QR, bit 2 = SMS.
  static String _labelForType(AppLocalizations l10n, String tipo) {
    if (tipo.length != 3) return tipo.isNotEmpty ? tipo : l10n.unknownType;
    final labels = <String>[];
    if (tipo[2] == '1') labels.add(l10n.typeDniNie);
    if (tipo[1] == '1') labels.add(l10n.typeQr);
    if (tipo[0] == '1') labels.add(l10n.typeSms);
    return labels.isNotEmpty ? labels.join(' + ') : tipo;
  }

  static IconData _iconForType(String tipo) {
    if (tipo.length != 3) return Icons.vpn_key;
    // Show the most specific icon.
    if (tipo[1] == '1') return Icons.qr_code;
    if (tipo[2] == '1') return Icons.badge;
    if (tipo[0] == '1') return Icons.sms;
    return Icons.vpn_key;
  }
}
