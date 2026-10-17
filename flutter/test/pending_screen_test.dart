import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:llave/l10n/app_localizations.dart';
import 'package:llave/screens/pending_screen.dart';
import 'package:llave/src/llave_bridge.dart';

const _peticion = {
  'tokenClaveMovil': 'tok-1',
  'codigoIdP': '7381',
  'organismo': 'Seguridad Social',
  'timestamp': '20261016T120000000Z',
};

/// Bridge that answers polls from a queue (then "nothing") and records
/// confirm/reject calls.
class _FakeBridge extends MockLlaveBridge {
  _FakeBridge(this.polls, {this.morePending = false, this.throwOnPoll = false});

  final List<Map<String, Object?>?> polls;
  final bool morePending;
  final bool throwOnPoll;
  int pollCount = 0;
  final confirmed = <String>[];
  final rejected = <String>[];

  @override
  Future<LlaveApiResult> getPendingRequests() async {
    if (throwOnPoll) throw Exception('native library failed');
    final next = pollCount < polls.length ? polls[pollCount] : null;
    pollCount++;
    return LlaveApiResult(ok: true, data: jsonEncode(next));
  }

  @override
  Future<LlaveApiResult> confirmRequest(String token, String idpCode) async {
    confirmed.add('$token/$idpCode');
    return LlaveApiResult(
      ok: true,
      data: jsonEncode({'status': 'OK', 'more_pending': morePending}),
    );
  }

  @override
  Future<LlaveApiResult> rejectRequest(String token, String idpCode) async {
    rejected.add('$token/$idpCode');
    return LlaveApiResult(
      ok: true,
      data: jsonEncode({'status': 'OK', 'more_pending': morePending}),
    );
  }
}

Future<void> _pump(WidgetTester tester, _FakeBridge bridge) {
  return tester.pumpWidget(
    ProviderScope(
      overrides: [llaveBridgeProvider.overrideWithValue(bridge)],
      child: const MaterialApp(
        locale: Locale('en'),
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: PendingScreen(
          pollInterval: Duration(seconds: 1),
          waitTimeout: Duration(seconds: 3),
        ),
      ),
    ),
  );
}

void main() {
  testWidgets('starts waiting as soon as it opens', (tester) async {
    await _pump(tester, _FakeBridge([]));
    await tester.pump();

    expect(find.text('Waiting for a login request…'), findsOneWidget);
  });

  testWidgets('shows a request that arrives on a later poll', (tester) async {
    await _pump(tester, _FakeBridge([null, _peticion]));
    await tester.pump();

    await tester.pump(const Duration(seconds: 1));
    await tester.pump();

    expect(find.text('Seguridad Social'), findsOneWidget);
    expect(find.text('7381'), findsOneWidget);
  });

  testWidgets('answering no rejects the request', (tester) async {
    final bridge = _FakeBridge([_peticion]);
    await _pump(tester, bridge);
    await tester.pump();

    await tester.tap(find.text('No, reject'));
    await tester.pump();

    expect(bridge.rejected, ['tok-1/7381']);
  });

  testWidgets('answering yes confirms the request', (tester) async {
    final bridge = _FakeBridge([_peticion]);
    await _pump(tester, bridge);
    await tester.pump();

    await tester.tap(find.text('Yes, confirm'));
    await tester.pump();

    expect(bridge.confirmed, ['tok-1/7381']);
  });

  testWidgets('keeps waiting after confirm when more requests are pending', (tester) async {
    final bridge = _FakeBridge([_peticion], morePending: true);
    await _pump(tester, bridge);
    await tester.pump();

    await tester.tap(find.text('Yes, confirm'));
    await tester.pump();

    expect(find.text('Waiting for a login request…'), findsOneWidget);
  });

  testWidgets('gives up after the wait timeout', (tester) async {
    await _pump(tester, _FakeBridge([]));
    await tester.pump();

    await tester.pump(const Duration(seconds: 4));

    expect(
      find.text('No request arrived. Start the login on the website and try again.'),
      findsOneWidget,
    );
  });

  testWidgets('cancel stops polling', (tester) async {
    final bridge = _FakeBridge([]);
    await _pump(tester, bridge);
    await tester.pump();

    await tester.tap(find.text('Cancel'));
    await tester.pump();
    final polls = bridge.pollCount;
    await tester.pump(const Duration(seconds: 2));

    expect(bridge.pollCount, polls);
    expect(find.text('Wait for a login'), findsOneWidget);
  });

  testWidgets('keeps waiting after a reject when more requests are pending', (tester) async {
    final bridge = _FakeBridge([_peticion], morePending: true);
    await _pump(tester, bridge);
    await tester.pump();

    await tester.tap(find.text('No, reject'));
    await tester.pump();

    expect(find.text('Waiting for a login request…'), findsOneWidget);
  });

  testWidgets('stops with a message when polling fails', (tester) async {
    await _pump(tester, _FakeBridge([], throwOnPoll: true));
    await tester.pump();
    await tester.pump();

    expect(find.text('Wait for a login'), findsOneWidget);
    expect(
      find.text('Something went wrong. Try again.'),
      findsOneWidget,
    );
  });
}
