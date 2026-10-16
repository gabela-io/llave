import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:llave/l10n/app_localizations.dart';
import 'package:llave/src/errors.dart';
import 'package:llave/src/llave_bridge.dart';

LlaveApiResult _failure(String code, {String? server, String error = 'English text'}) =>
    LlaveApiResult(ok: false, data: '', error: error, errorCode: code, serverMessage: server);

void main() {
  final l10n = lookupAppLocalizations(const Locale('en'));

  test('known kinds get a localised message', () {
    expect(friendlyError(l10n, _failure('device_not_active')), l10n.errorDeviceNotActive);
    expect(friendlyError(l10n, _failure('http')), l10n.errorNoConnection);
    expect(friendlyError(l10n, _failure('no_pending_pin')), l10n.pinNoPending);
  });

  test('a server KO shows the AEAT message', () {
    expect(
      friendlyError(l10n, _failure('api', server: 'Datos de operación no correctos')),
      'Datos de operación no correctos',
    );
  });

  test('a server KO without a message is an unexpected response', () {
    expect(friendlyError(l10n, _failure('api')), l10n.errorUnexpectedResponse);
  });

  test('unknown kinds fall back to the error text', () {
    expect(friendlyError(l10n, _failure('keyring', error: 'boom')), 'boom');
    expect(
      friendlyError(l10n, const LlaveApiResult(ok: false, data: '')),
      l10n.unknownError,
    );
  });
}
