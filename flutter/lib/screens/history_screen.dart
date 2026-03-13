import 'dart:convert';

import 'package:flutter/material.dart';

import '../src/ip_rate_limit.dart';
import '../src/rust/api/api.dart' as native_ffi;

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

    try {
      final result = await native_ffi.getHistory();
      if (!result.ok) {
        final err = result.error ?? 'Unknown error';
        setState(() => _error = err);
        if (mounted && isIpRateLimited(err)) {
          showIpRateLimitBanner(context);
        }
        return;
      }
      final parsed = jsonDecode(result.data);
      final List<Map<String, dynamic>> items = [];
      // The respuesta is a direct JSON array of operation entries.
      if (parsed is List) {
        for (final item in parsed) {
          if (item is Map<String, dynamic>) items.add(item);
        }
      }
      setState(() => _operations = items);
    } catch (e) {
      final err = e.toString();
      setState(() => _error = err);
      if (mounted && isIpRateLimited(err)) {
        showIpRateLimitBanner(context);
      }
    } finally {
      if (mounted) setState(() => _loading = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('History'),
        actions: [
          IconButton(
            onPressed: _loading ? null : _load,
            icon: const Icon(Icons.refresh),
            tooltip: 'Refresh',
          ),
        ],
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : _error != null
              ? Center(
                  child: Padding(
                    padding: const EdgeInsets.all(24),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(Icons.error_outline,
                            size: 48, color: theme.colorScheme.error),
                        const SizedBox(height: 16),
                        Text('Error: $_error', textAlign: TextAlign.center),
                        const SizedBox(height: 16),
                        FilledButton(
                            onPressed: _load, child: const Text('Retry')),
                      ],
                    ),
                  ),
                )
              : _operations.isEmpty
                  ? Center(
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Icon(Icons.history,
                              size: 48,
                              color: theme.colorScheme.onSurfaceVariant
                                  .withValues(alpha: 0.4)),
                          const SizedBox(height: 16),
                          Text('No operations found',
                              style: theme.textTheme.titleMedium),
                          const SizedBox(height: 16),
                          FilledButton(
                              onPressed: _load, child: const Text('Refresh')),
                        ],
                      ),
                    )
                  : RefreshIndicator(
                      onRefresh: _load,
                      child: ListView.builder(
                        padding: const EdgeInsets.all(16),
                        itemCount: _operations.length,
                        itemBuilder: (context, index) =>
                            _OperationCard(operation: _operations[index]),
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
    final theme = Theme.of(context);
    final tipo = operation['tipo']?.toString() ?? '';
    final resultado = operation['resultado']?.toString() ?? '';
    final organismo = operation['organismo']?.toString() ?? '';
    final fecha = operation['fecha']?.toString() ?? '';

    // "01" = confirmed/accepted, "10" = rejected.
    final isAccepted = resultado == '01';
    final resultLabel = isAccepted ? 'Accepted' : 'Rejected';
    final resultColor = isAccepted ? Colors.green : theme.colorScheme.error;

    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: ListTile(
        leading: Icon(
          _iconForType(tipo),
          color: resultColor,
        ),
        title: Text(
            organismo.isNotEmpty ? organismo : _labelForType(tipo)),
        subtitle: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(_labelForType(tipo), style: theme.textTheme.bodySmall),
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
  static String _labelForType(String tipo) {
    if (tipo.length != 3) return tipo.isNotEmpty ? tipo : 'Unknown';
    final labels = <String>[];
    if (tipo[2] == '1') labels.add('DNI/NIE');
    if (tipo[1] == '1') labels.add('QR');
    if (tipo[0] == '1') labels.add('SMS');
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
