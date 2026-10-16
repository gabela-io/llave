import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:llave/l10n/app_localizations.dart';
import 'package:llave/screens/dni_auth_screen.dart';
import 'package:llave/src/llave_bridge.dart';

/// Bridge that records phase-1 calls and can ask for the date of birth once.
class _FakeBridge extends MockLlaveBridge {
  _FakeBridge({this.challenge});

  /// When set, the first call fails with this date-of-birth challenge.
  final String? challenge;
  final calls = <(String, String, String, String)>[];

  @override
  Future<LlaveApiResult> dniAuthenticate(String nif, String fecha, String soporte,
      {String challenge = ''}) async {
    calls.add((nif, fecha, soporte, challenge));
    if (this.challenge != null && calls.length == 1) {
      return LlaveApiResult(
        ok: false,
        data: '{"challenge":"${this.challenge}"}',
        error: 'DNI/NIE check needs your date of birth',
        errorCode: 'dni_birth_date_required',
        serverMessage: 'Introduzca su fecha de nacimiento',
      );
    }
    return const LlaveApiResult(
      ok: true,
      data: '{"movil":"***1234","cookies_json":"[]",'
          '"timestamp_alta_sms":"ts","token_clave_movil_sms":"tok"}',
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
        home: DniAuthScreen(),
      ),
    ),
  );
}

Finder _field(String label) => find.widgetWithText(TextFormField, label);

Future<void> _submit(WidgetTester tester) async {
  await tester.tap(find.text('Authenticate'));
  await tester.pump();
  await tester.pump();
}

void main() {
  testWidgets('a DNI asks for the validity date and sends it as DD-MM-YYYY',
      (tester) async {
    final bridge = _FakeBridge();
    await _pump(tester, bridge);

    await tester.enterText(_field('NIF (DNI/NIE)'), '12345678Z');
    await tester.pump();
    expect(_field('Support number'), findsNothing);
    await tester.enterText(_field('ID card validity date'), '01-02-2030');
    await _submit(tester);

    expect(bridge.calls, [('12345678Z', '01-02-2030', '', '')]);
  });

  testWidgets('a NIE X/Y/Z asks for the support number only', (tester) async {
    final bridge = _FakeBridge();
    await _pump(tester, bridge);

    await tester.enterText(_field('NIF (DNI/NIE)'), 'X1234567L');
    await tester.pump();
    expect(_field('ID card validity date'), findsNothing);
    await tester.enterText(_field('Support number'), 'E12345678');
    await _submit(tester);

    expect(bridge.calls, [('X1234567L', '', 'E12345678', '')]);
  });

  testWidgets('a NIE M asks for the date of birth', (tester) async {
    final bridge = _FakeBridge();
    await _pump(tester, bridge);

    await tester.enterText(_field('NIF (DNI/NIE)'), 'M1234567L');
    await tester.pump();
    await tester.enterText(_field('Date of birth'), '03-04-1980');
    await _submit(tester);

    expect(bridge.calls, [('M1234567L', '03-04-1980', '', '')]);
  });

  testWidgets('retries with the date of birth when the server asks for it',
      (tester) async {
    final bridge = _FakeBridge(challenge: 'fecha_nie');
    await _pump(tester, bridge);

    await tester.enterText(_field('NIF (DNI/NIE)'), 'X1234567L');
    await tester.pump();
    await tester.enterText(_field('Support number'), 'E12345678');
    await _submit(tester);

    expect(find.text('Introduzca su fecha de nacimiento'), findsOneWidget);
    await tester.enterText(_field('Date of birth'), '03-04-1980');
    await _submit(tester);

    expect(bridge.calls.last, ('X1234567L', '03-04-1980', '', 'fecha_nie'));
    expect(find.text('***1234'), findsOneWidget);
  });
}
