import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:llave/l10n/app_localizations.dart';
import 'package:llave/src/llave_bridge.dart';
import 'package:llave/widgets/pin_card.dart';

/// Bridge whose [requestPin] answers with a fixed result or error.
class _FakeBridge extends MockLlaveBridge {
  _FakeBridge({this.result, this.error});

  final LlavePinResult? result;
  final Object? error;
  int calls = 0;

  @override
  Future<LlavePinResult> requestPin() async {
    calls++;
    if (error != null) throw error!;
    return result!;
  }
}

const _result = LlavePinResult(pin: '424242', timeToLiveSeconds: '3', nif: '12345678Z');

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
    await _pump(tester, _FakeBridge(result: _result));

    expect(find.text('Show PIN'), findsOneWidget);
    expect(find.text('424242'), findsNothing);
  });

  testWidgets('reveals the pending PIN with its validity', (tester) async {
    await _pump(tester, _FakeBridge(result: _result));

    await tester.tap(find.text('Show PIN'));
    await tester.pump();

    expect(find.text('424242'), findsOneWidget);
    expect(find.text('Valid for 3 s'), findsOneWidget);
  });

  testWidgets('counts down and hides the PIN when it expires', (tester) async {
    await _pump(tester, _FakeBridge(result: _result));
    await tester.tap(find.text('Show PIN'));
    await tester.pump();

    await tester.pump(const Duration(seconds: 1));
    expect(find.text('Valid for 2 s'), findsOneWidget);

    await tester.pump(const Duration(seconds: 2));
    expect(find.text('424242'), findsNothing);
    expect(find.text('Show PIN'), findsOneWidget);
  });

  testWidgets('hide button hides the PIN', (tester) async {
    await _pump(tester, _FakeBridge(result: _result));
    await tester.tap(find.text('Show PIN'));
    await tester.pump();

    await tester.tap(find.text('Hide'));
    await tester.pump();

    expect(find.text('424242'), findsNothing);
  });

  testWidgets('explains when there is no pending PIN', (tester) async {
    final bridge = _FakeBridge(
      error: Exception('No pending Cl@ve PIN. Start a PIN request on the AEAT website'),
    );
    await _pump(tester, bridge);

    await tester.tap(find.text('Show PIN'));
    await tester.pump();

    expect(
      find.text('There is no pending PIN. Request one on the AEAT website first.'),
      findsOneWidget,
    );
  });

  testWidgets('does not request the PIN when presence is not confirmed', (tester) async {
    final bridge = _FakeBridge(result: _result);
    await _pump(tester, bridge, confirmPresence: () async => false);

    await tester.tap(find.text('Show PIN'));
    await tester.pump();

    expect(bridge.calls, 0);
    expect(find.text('424242'), findsNothing);
  });
}
