import 'package:flutter_test/flutter_test.dart';
import 'package:llave/src/auth_provider.dart';
import 'package:llave/src/llave_bridge.dart';

/// Bridge whose server refuses the deactivation, recording local logouts.
class _RefusingBridge extends MockLlaveBridge {
  int logouts = 0;

  @override
  Future<LlaveApiResult> deactivate() async => const LlaveApiResult(
        ok: false,
        data: '',
        error: 'HTTP request failed',
        errorCode: 'http',
      );

  @override
  bool logout() {
    logouts++;
    return true;
  }
}

void main() {
  test('a refused deactivation keeps the local credentials', () async {
    final bridge = _RefusingBridge();
    final auth = AuthNotifier(bridge);

    final result = await auth.deactivate();

    expect(result.ok, isFalse);
    expect(bridge.logouts, 0);
    expect(auth.state, isNot(isA<AuthUnauthenticated>()));
  });

  test('only a revoked device is forgotten on failure', () async {
    final bridge = _RefusingBridge();
    final auth = AuthNotifier(bridge);

    await auth.reportFailure(
      const LlaveApiResult(ok: false, data: '', errorCode: 'device_not_active'),
    );

    expect(bridge.logouts, 0);
  });
}
