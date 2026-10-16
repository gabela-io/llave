import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/app_localizations.dart';
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
    this.pollInterval = const Duration(seconds: 3),
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
    final result = await ref.read(llaveBridgeProvider).getPendingRequests();
    if (!mounted || generation != _generation) return;

    if (!result.ok) {
      _stopWaiting(friendlyError(AppLocalizations.of(context)!, result));
      return;
    }
    final peticion = jsonDecode(result.data);
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
    final result = confirm
        ? await bridge.confirmRequest(token, code)
        : await bridge.rejectRequest(token, code);
    if (!mounted) return;
    setState(() => _busy = false);

    if (!result.ok) {
      messenger.showSnackBar(
        SnackBar(content: Text(friendlyError(l10n, result))),
      );
      return;
    }
    messenger.showSnackBar(
      SnackBar(content: Text(confirm ? l10n.requestConfirmed : l10n.requestRejected)),
    );

    // The server says whether another request is already queued.
    final data = jsonDecode(result.data);
    final morePending = data is Map && data['pending_requests'] == 'S';
    if (confirm && morePending) {
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
