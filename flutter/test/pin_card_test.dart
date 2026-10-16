import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:llave/l10n/app_localizations.dart';
import 'package:llave/src/llave_bridge.dart';
import 'package:llave/widgets/pin_card.dart';

/// Bridge whose [requestPin] answers with a fixed result.
class _FakeBridge extends MockLlaveBridge {
  _FakeBridge({LlaveApiResult? result}) : result = result ?? _result;

  final LlaveApiResult result;
  int calls = 0;

  @override
  Future<LlaveApiResult> requestPin() async {
    calls++;
    return result;
  }
}

LlaveApiResult _pin(String ttl) => LlaveApiResult(
      ok: true,
      data: '{"pin":"424242","time_to_live_seconds":"$ttl","nif":"12345678Z"}',
    );

final _result = _pin('3');

LlaveApiResult _failure(String code) =>
    LlaveApiResult(ok: false, data: '', error: 'English text', errorCode: code);

Future<void> _pump(
  WidgetTester tester,
  _FakeBridge bridge, {
  Future<bool> Function()? confirmPresence,
}) {
  return tester.pumpWidget(
    ProviderScope(
      overrides: [llaveBridgeProvider.overrideWithValue(bridge)],
      child: MaterialApp(
        locale: const Locale('en'),
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: Scaffold(
          body: PinCard(confirmPresence: confirmPresence ?? () async => true),
        ),
      ),
    ),
  );
}

void main() {
  testWidgets('starts hidden with a reveal button', (tester) async {
    await _pump(tester, _FakeBridge());

    expect(find.text('Show PIN'), findsOneWidget);
    expect(find.text('424242'), findsNothing);
  });

  testWidgets('reveals the pending PIN with its validity', (tester) async {
    await _pump(tester, _FakeBridge());

    await tester.tap(find.text('Show PIN'));
    await tester.pump();

    expect(find.text('424242'), findsOneWidget);
    expect(find.text('Valid for 3 s'), findsOneWidget);
  });

  testWidgets('counts down and hides the PIN when it expires', (tester) async {
    await _pump(tester, _FakeBridge());
    await tester.tap(find.text('Show PIN'));
    await tester.pump();

    await tester.pump(const Duration(seconds: 1));
    expect(find.text('Valid for 2 s'), findsOneWidget);

    await tester.pump(const Duration(seconds: 2));
    expect(find.text('424242'), findsNothing);
    expect(find.text('Show PIN'), findsOneWidget);
  });

  testWidgets('hide button hides the PIN', (tester) async {
    await _pump(tester, _FakeBridge());
    await tester.tap(find.text('Show PIN'));
    await tester.pump();

    await tester.tap(find.text('Hide'));
    await tester.pump();

    expect(find.text('424242'), findsNothing);
  });

  testWidgets('explains when there is no pending PIN', (tester) async {
    final bridge = _FakeBridge(result: _failure('no_pending_pin'));
    await _pump(tester, bridge);

    await tester.tap(find.text('Show PIN'));
    await tester.pump();

    expect(
      find.text('There is no pending PIN. Request one on the AEAT website first.'),
      findsOneWidget,
    );
  });

  testWidgets('does not request the PIN when presence is not confirmed', (tester) async {
    final bridge = _FakeBridge();
    await _pump(tester, bridge, confirmPresence: () async => false);

    await tester.tap(find.text('Show PIN'));
    await tester.pump();

    expect(bridge.calls, 0);
    expect(find.text('424242'), findsNothing);
  });

  testWidgets('keeps the PIN visible when the server sent no validity', (tester) async {
    await _pump(tester, _FakeBridge(result: _pin('')));
    await tester.tap(find.text('Show PIN'));
    await tester.pump();

    await tester.pump(const Duration(seconds: 5));
    expect(find.text('424242'), findsOneWidget);
  });

  testWidgets('shows a readable message on a connection error', (tester) async {
    await _pump(tester, _FakeBridge(result: _failure('http')));
    await tester.tap(find.text('Show PIN'));
    await tester.pump();

    expect(
      find.text('Could not reach the AEAT servers. Check your connection and try again.'),
      findsOneWidget,
    );
  });
}
