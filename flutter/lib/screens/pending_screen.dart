import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/app_localizations.dart';
import '../src/api_helpers.dart';
import '../src/auth_provider.dart';
import '../src/errors.dart';
import '../src/llave_bridge.dart';
import '../widgets/request_card.dart';

enum _Phase { idle, waiting, found }

/// "Waiting for a login": polls for a Cl@ve Móvil request for a short while
/// after the user starts a login on a website, then shows it with the app's
/// security check (organismo, verification code, "did you start this?").
///
/// Replaces the official app's push notification, which only wakes the app
/// up to run this same query.
class PendingScreen extends ConsumerStatefulWidget {
  const PendingScreen({
    super.key,
    // The CLI's default interval; about 24 queries per wait.
    this.pollInterval = const Duration(seconds: 5),
    this.waitTimeout = const Duration(minutes: 2),
  });

  final Duration pollInterval;
  final Duration waitTimeout;

  @override
  ConsumerState<PendingScreen> createState() => _PendingScreenState();
}

class _PendingScreenState extends ConsumerState<PendingScreen> {
  _Phase _phase = _Phase.idle;
  Map<String, dynamic>? _request;
  String? _message;
  bool _busy = false;
  int _remaining = 0;

  Timer? _pollTimer;
  Timer? _ticker;

  /// Bumped on every start/stop so late poll results are ignored.
  int _generation = 0;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _startWaiting());
  }

  @override
  void dispose() {
    _stopTimers();
    super.dispose();
  }

  void _stopTimers() {
    _generation++;
    _pollTimer?.cancel();
    _ticker?.cancel();
  }

  void _startWaiting() {
    if (!mounted) return;
    _stopTimers();
    final generation = _generation;
    setState(() {
      _phase = _Phase.waiting;
      _request = null;
      _message = null;
      _remaining = widget.waitTimeout.inSeconds;
    });
    _ticker = Timer.periodic(const Duration(seconds: 1), (_) {
      if (_remaining <= 1) {
        _stopWaiting(AppLocalizations.of(context)!.waitTimeout);
      } else {
        setState(() => _remaining--);
      }
    });
    _poll(generation);
  }

  void _stopWaiting([String? message]) {
    _stopTimers();
    if (!mounted) return;
    setState(() {
      _phase = _Phase.idle;
      _message = message;
    });
  }

  Future<void> _poll(int generation) async {
    LlaveApiResult result;
    try {
      result = await ref.read(llaveBridgeProvider).getPendingRequests();
    } catch (e) {
      result = LlaveApiResult.exception(e);
    }
    if (!mounted || generation != _generation) return;

    final l10n = AppLocalizations.of(context)!;
    if (!result.ok) {
      _stopWaiting(friendlyError(l10n, result));
      await ref.read(authProvider.notifier).reportFailure(result);
      return;
    }
    // `data` is the pending request, or `null` while there is none.
    final peticion = tryJsonDecode(result.data);
    if (peticion is Map<String, dynamic>) {
      _stopTimers();
      setState(() {
        _phase = _Phase.found;
        _request = peticion;
      });
      return;
    }
    _pollTimer = Timer(widget.pollInterval, () => _poll(generation));
  }

  Future<void> _answer({required bool confirm}) async {
    final request = _request;
    if (request == null) return;
    final l10n = AppLocalizations.of(context)!;
    final messenger = ScaffoldMessenger.of(context);
    final token = request['tokenClaveMovil']?.toString() ?? '';
    final code = request['codigoIdP']?.toString() ?? '';

    setState(() => _busy = true);
    final bridge = ref.read(llaveBridgeProvider);
    LlaveApiResult result;
    try {
      result = confirm
          ? await bridge.confirmRequest(token, code)
          : await bridge.rejectRequest(token, code);
    } catch (e) {
      result = LlaveApiResult.exception(e);
    }
    if (!mounted) return;
    setState(() => _busy = false);

    if (!result.ok) {
      messenger.showSnackBar(
        SnackBar(content: Text(friendlyError(l10n, result))),
      );
      await ref.read(authProvider.notifier).reportFailure(result);
      return;
    }
    messenger.showSnackBar(
      SnackBar(content: Text(confirm ? l10n.requestConfirmed : l10n.requestRejected)),
    );

    // Like the app, look for the next request after a confirm or a reject
    // unless the server says there is none (`more_pending`, from the core).
    final data = tryJsonDecode(result.data);
    final morePending = data is! Map || data['more_pending'] != false;
    if (morePending) {
      _startWaiting();
    } else {
      _stopWaiting();
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return Scaffold(
      appBar: AppBar(title: Text(l10n.pendingTitle)),
      body: SafeArea(
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: switch (_phase) {
              _Phase.idle => _Idle(message: _message, onStart: _startWaiting),
              _Phase.waiting => _Waiting(remaining: _remaining, onCancel: _stopWaiting),
              _Phase.found => RequestCard(
                  request: _request!,
                  busy: _busy,
                  onConfirm: () => _answer(confirm: true),
                  onReject: () => _answer(confirm: false),
                ),
            },
          ),
        ),
      ),
    );
  }
}

class _Idle extends StatelessWidget {
  const _Idle({required this.message, required this.onStart});

  final String? message;
  final VoidCallback onStart;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(Icons.login, size: 56, color: theme.colorScheme.onSurfaceVariant),
        const SizedBox(height: 16),
        Text(
          message ?? l10n.waitHint,
          textAlign: TextAlign.center,
          style: theme.textTheme.bodyLarge,
        ),
        const SizedBox(height: 24),
        FilledButton.icon(
          onPressed: onStart,
          icon: const Icon(Icons.hourglass_top),
          label: Text(l10n.waitStart),
        ),
      ],
    );
  }
}

class _Waiting extends StatelessWidget {
  const _Waiting({required this.remaining, required this.onCancel});

  final int remaining;
  final VoidCallback onCancel;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const CircularProgressIndicator(),
        const SizedBox(height: 24),
        Text(l10n.waitSearching, style: theme.textTheme.titleMedium),
        const SizedBox(height: 8),
        Text(
          l10n.waitRemaining(remaining),
          style: theme.textTheme.bodyMedium?.copyWith(
            color: theme.colorScheme.onSurfaceVariant,
          ),
        ),
        const SizedBox(height: 24),
        TextButton(onPressed: onCancel, child: Text(l10n.cancel)),
      ],
    );
  }
}
