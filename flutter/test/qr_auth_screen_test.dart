import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:llave/l10n/app_localizations.dart';
import 'package:llave/screens/qr_auth_screen.dart';
import 'package:llave/src/llave_bridge.dart';

/// Bridge whose QR scan creates one request and records the answer.
class _FakeBridge extends MockLlaveBridge {
  final confirmed = <String>[];
  final rejected = <String>[];

  @override
  Future<LlaveApiResult> qrAuthenticate(String value) async {
    return LlaveApiResult(
      ok: true,
      data: jsonEncode({
        'peticion': {
          'tokenClaveMovil': 'tok-qr',
          'codigoIdP': '5521',
          'organismo': 'DGT',
        },
      }),
    );
  }

  @override
  Future<LlaveApiResult> confirmRequest(String token, String idpCode) async {
    confirmed.add('$token/$idpCode');
    return const LlaveApiResult(ok: true, data: '{"status":"OK"}');
  }

  @override
  Future<LlaveApiResult> rejectRequest(String token, String idpCode) async {
    rejected.add('$token/$idpCode');
    return const LlaveApiResult(ok: true, data: '{"status":"OK"}');
  }
}

/// Opens the screen (manual input on desktop) and submits a QR value.
Future<void> _scan(WidgetTester tester, _FakeBridge bridge) async {
  await tester.pumpWidget(
    ProviderScope(
      overrides: [llaveBridgeProvider.overrideWithValue(bridge)],
      child: const MaterialApp(
        locale: Locale('en'),
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: QrAuthScreen(),
      ),
    ),
  );
  await tester.enterText(find.byType(TextField), '0${'A' * 160}');
  await tester.testTextInput.receiveAction(TextInputAction.done);
  await tester.pump();
}

void main() {
  testWidgets('shows the security check for the scanned request', (tester) async {
    await _scan(tester, _FakeBridge());

    expect(find.text('DGT'), findsOneWidget);
    expect(find.text('5521'), findsOneWidget);
    expect(find.text('Yes, confirm'), findsOneWidget);
    expect(find.text('No, reject'), findsOneWidget);
  });

  testWidgets('answering yes confirms and reports success', (tester) async {
    final bridge = _FakeBridge();
    await _scan(tester, bridge);

    await tester.tap(find.text('Yes, confirm'));
    await tester.pump();

    expect(bridge.confirmed, ['tok-qr/5521']);
    expect(find.text('QR authentication successful'), findsOneWidget);
  });

  testWidgets('answering no rejects and returns to scanning', (tester) async {
    final bridge = _FakeBridge();
    await _scan(tester, bridge);

    await tester.tap(find.text('No, reject'));
    await tester.pump();

    expect(bridge.rejected, ['tok-qr/5521']);
    expect(bridge.confirmed, isEmpty);
    expect(find.byType(TextField), findsOneWidget);
  });
}
