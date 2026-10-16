import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../l10n/app_localizations.dart';
import '../src/api_helpers.dart';
import '../src/auth_provider.dart';
import '../src/llave_bridge.dart';
import '../src/routes.dart';
import '../widgets/pin_card.dart';

class HomeScreen extends ConsumerWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final session = ref.watch(currentSessionProvider);

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.homeTitle),
        centerTitle: true,
        actions: [
          if (session != null)
            IconButton(
              icon: const Icon(Icons.logout),
              tooltip: l10n.logOut,
              onPressed: () => _confirmLogout(context, ref),
            ),
        ],
      ),
      // The router only shows Home with an active session.
      body: session == null
          ? const SizedBox.shrink()
          : _AuthenticatedBody(session: session),
    );
  }

  void _confirmLogout(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(l10n.logOutTitle),
        content: Text(l10n.logOutMessage),
        actions: [
          TextButton(
              onPressed: () => Navigator.pop(ctx), child: Text(l10n.cancel)),
          FilledButton(
            onPressed: () {
              Navigator.pop(ctx);
              ref.read(authProvider.notifier).logout();
            },
            child: Text(l10n.logOut),
          ),
        ],
      ),
    );
  }
}

class _AuthenticatedBody extends ConsumerStatefulWidget {
  final dynamic session;
  const _AuthenticatedBody({required this.session});

  @override
  ConsumerState<_AuthenticatedBody> createState() => _AuthenticatedBodyState();
}

class _AuthenticatedBodyState extends ConsumerState<_AuthenticatedBody> {
  bool _loading = false;
  Map<String, dynamic>? _accountData;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadAccountData();
  }

  Future<void> _loadAccountData() async {
    setState(() {
      _loading = true;
      _error = null;
    });

    await handleApiCall(
      apiCall: () => ref.read(llaveBridgeProvider).getMyData(),
      setLoading: (v) {
        if (mounted) setState(() => _loading = v);
      },
      setError: (e) {
        if (mounted) setState(() => _error = e);
      },
      context: context,
      onData: (decoded) {
        setState(() =>
            _accountData = decoded is Map<String, dynamic> ? decoded : null);
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        _AccountCard(
          session: widget.session,
          accountData: _accountData,
          loading: _loading,
          error: _error,
          onRetry: _loadAccountData,
        ),
        const SizedBox(height: 16),
        const PinCard(),
        const SizedBox(height: 20),
        SizedBox(
          height: 56,
          child: FilledButton.icon(
            onPressed: () => context.go(AppRoutes.qr),
            icon: const Icon(Icons.qr_code_2, size: 24),
            label: Text(
              l10n.qrAuthentication,
              style: theme.textTheme.titleMedium?.copyWith(
                color: theme.colorScheme.onPrimary,
              ),
            ),
          ),
        ),
      ],
    );
  }
}

class _AccountCard extends StatelessWidget {
  final dynamic session;
  final Map<String, dynamic>? accountData;
  final bool loading;
  final String? error;
  final VoidCallback onRetry;

  const _AccountCard({
    required this.session,
    required this.accountData,
    required this.loading,
    required this.error,
    required this.onRetry,
  });

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(Icons.person, color: theme.colorScheme.primary),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(session.nif,
                          style: theme.textTheme.titleMedium
                              ?.copyWith(fontWeight: FontWeight.bold)),
                      Text(
                        l10n.activeSince(_formatDate(session.createdAt)),
                        style: theme.textTheme.bodySmall?.copyWith(
                          color: theme.colorScheme.onSurfaceVariant,
                        ),
                      ),
                    ],
                  ),
                ),
                if (loading)
                  const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                else
                  InkWell(
                    borderRadius: BorderRadius.circular(16),
                    onTap: onRetry,
                    child: Icon(
                      Icons.refresh,
                      size: 20,
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                  ),
              ],
            ),
            if (accountData != null) ...[
              const SizedBox(height: 12),
              if (accountData!['email'] != null)
                _AccountRow(label: l10n.emailLabel, value: accountData!['email'].toString()),
              if (accountData!['numTelefono'] != null)
                _AccountRow(label: l10n.phoneLabel, value: accountData!['numTelefono'].toString()),
              if (accountData!['nivelRegistro'] != null)
                _AccountRow(
                    label: l10n.levelLabel,
                    value: _registrationLevel(l10n, accountData!['nivelRegistro'].toString())),
            ],
            if (error != null && accountData == null)
              Padding(
                padding: const EdgeInsets.only(top: 4),
                child: Text(
                  error!,
                  style: theme.textTheme.bodySmall?.copyWith(
                    color: theme.colorScheme.error,
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }

  static String _registrationLevel(AppLocalizations l10n, String level) => switch (level) {
        '1' => l10n.registrationBasic,
        '2' => l10n.registrationAdvanced,
        '3' => l10n.registrationSuperior,
        _ => level,
      };

  static String _formatDate(String iso) {
    try {
      final dt = DateTime.parse(iso);
      return '${dt.day.toString().padLeft(2, '0')}/'
          '${dt.month.toString().padLeft(2, '0')}/'
          '${dt.year} '
          '${dt.hour.toString().padLeft(2, '0')}:'
          '${dt.minute.toString().padLeft(2, '0')}';
    } catch (_) {
      return iso;
    }
  }
}

class _AccountRow extends StatelessWidget {
  final String label;
  final String value;
  const _AccountRow({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 64,
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
    );
  }
}
