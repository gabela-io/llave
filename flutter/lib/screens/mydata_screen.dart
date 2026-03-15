import 'package:flutter/material.dart';
import '../l10n/app_localizations.dart';

import '../src/api_helpers.dart';
import '../src/llave_bridge.dart';
import '../src/rust/api/api.dart' as native_ffi;
import '../widgets/data_screen_scaffold.dart';

class MyDataScreen extends StatefulWidget {
  const MyDataScreen({super.key});

  @override
  State<MyDataScreen> createState() => _MyDataScreenState();
}

class _MyDataScreenState extends State<MyDataScreen> {
  bool _loading = false;
  Map<String, dynamic>? _data;
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
        final r = await native_ffi.getMyData();
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
        setState(() =>
            _data = decoded is Map<String, dynamic> ? decoded : {'raw': decoded});
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.myDataTitle),
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
        isEmpty: _data == null,
        onRetry: _load,
        emptyIcon: Icons.person_outline,
        emptyTitle: l10n.noData,
        child: RefreshIndicator(
          onRefresh: _load,
          child: ListView(
            padding: const EdgeInsets.all(16),
            children: [
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(20),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Icon(Icons.person,
                              color: theme.colorScheme.primary),
                          const SizedBox(width: 12),
                          Text(l10n.accountInformation,
                              style: theme.textTheme.titleMedium),
                        ],
                      ),
                      const SizedBox(height: 16),
                      ..._buildFields(l10n, theme),
                    ],
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  List<Widget> _buildFields(AppLocalizations l10n, ThemeData theme) {
    if (_data == null) return [];

    // Fields from ClaveCheckMyDataSv + session NIF.
    final labels = {
      'nif': l10n.nifLabel,
      'email': l10n.emailLabel,
      'numTelefono': l10n.phoneLabel,
      'nivelRegistro': l10n.registrationLevel,
    };

    final widgets = <Widget>[];
    for (final entry in _data!.entries) {
      if (entry.value == null) continue;
      final label = labels[entry.key] ?? entry.key;
      final value = entry.value.toString();
      widgets.add(Padding(
        padding: const EdgeInsets.only(bottom: 12),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            SizedBox(
              width: 120,
              child: Text(
                label,
                style: theme.textTheme.bodySmall?.copyWith(
                  color: theme.colorScheme.onSurfaceVariant,
                ),
              ),
            ),
            Expanded(
              child: Text(value, style: theme.textTheme.bodyMedium),
            ),
          ],
        ),
      ));
    }
    return widgets;
  }
}
