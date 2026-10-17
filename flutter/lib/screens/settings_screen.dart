import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import '../l10n/app_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../src/auth_provider.dart';
import '../src/biometric_service.dart';
import '../src/errors.dart';
import '../src/routes.dart';
import '../src/llave_bridge.dart';
import '../src/proxy_provider.dart';
import '../src/theme_provider.dart';

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);
    final themeMode = ref.watch(themeModeProvider);

    return Scaffold(
      appBar: AppBar(title: Text(l10n.settingsTitle)),
      body: ListView(
        children: [
          const SizedBox(height: 8),
          _SectionHeader(l10n.appearance),
          ListTile(
            leading: Icon(_themeModeIcon(themeMode)),
            title: Text(l10n.theme),
            subtitle: Text(_themeModeLabel(l10n, themeMode)),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => _showThemePicker(context, ref),
          ),
          const Divider(),
          _SectionHeader(l10n.security),
          ListTile(
            leading: const Icon(Icons.pin_outlined),
            title: Text(l10n.changePinTitle),
            subtitle: Text(l10n.updatePinSubtitle),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => _showChangePinDialog(context, ref),
          ),
          ref.watch(biometricStateProvider).when(
                data: (state) {
                  if (!state.available) return const SizedBox.shrink();
                  return SwitchListTile(
                    secondary: const Icon(Icons.fingerprint),
                    title: Text(l10n.biometricUnlock),
                    subtitle: Text(l10n.biometricSubtitle),
                    value: state.enabled,
                    onChanged: (value) async {
                      if (value) {
                        await _enableBiometrics(context, ref);
                      } else {
                        await BiometricService.disable();
                      }
                      ref.invalidate(biometricStateProvider);
                    },
                  );
                },
                loading: () => const SizedBox.shrink(),
                error: (_, __) => const SizedBox.shrink(),
              ),
          const Divider(),
          _SectionHeader(l10n.network),
          _ProxyTile(),
          const Divider(),
          _SectionHeader(l10n.device),
          ListTile(
            leading: const Icon(Icons.delete_outline),
            title: Text(l10n.clearSession),
            subtitle: Text(l10n.clearSessionSubtitle),
            onTap: () async {
              await ref.read(authProvider.notifier).logout();
              if (context.mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(content: Text(l10n.sessionCleared)),
                );
              }
            },
          ),
          ListTile(
            leading: Icon(Icons.logout, color: theme.colorScheme.error),
            title: Text(l10n.deactivateDevice, style: TextStyle(color: theme.colorScheme.error)),
            subtitle: Text(l10n.deactivateSubtitle),
            onTap: () {
              showDialog(
                context: context,
                builder: (ctx) => AlertDialog(
                  title: Text(l10n.deactivateTitle),
                  content: Text(l10n.deactivateMessage),
                  actions: [
                    TextButton(onPressed: () => Navigator.pop(ctx), child: Text(l10n.cancel)),
                    FilledButton(
                      onPressed: () async {
                        Navigator.pop(ctx);
                        final messenger = ScaffoldMessenger.of(context);
                        LlaveApiResult result;
                        try {
                          result = await ref.read(authProvider.notifier).deactivate();
                        } catch (e) {
                          result = LlaveApiResult.exception(e);
                        }
                        // On failure the device stays active: keep the
                        // credentials and say why, so the user can retry.
                        if (!result.ok && result.errorCode != AuthNotifier.deviceRevokedCode) {
                          messenger.showSnackBar(
                            SnackBar(content: Text(friendlyError(l10n, result))),
                          );
                        }
                      },
                      child: Text(l10n.deactivate),
                    ),
                  ],
                ),
              );
            },
          ),
          const Divider(),
          _SectionHeader(l10n.about),
          ListTile(
            leading: const Icon(Icons.info_outline),
            title: Text(l10n.version),
            subtitle: Text(ref.read(llaveBridgeProvider).version()),
          ),
          ListTile(
            leading: const Icon(Icons.code),
            title: Text(l10n.poweredBy),
            subtitle: Text(l10n.poweredByValue),
          ),
          if (kDebugMode || kProfileMode) ...[
            const Divider(),
            _SectionHeader(l10n.developer),
            ListTile(
              leading: Icon(
                Icons.science,
                color: ref.watch(llaveBridgeProvider) is MockLlaveBridge
                    ? Colors.orange
                    : Colors.green,
              ),
              title: Text(l10n.developerTools),
              subtitle: Text(
                ref.watch(llaveBridgeProvider) is MockLlaveBridge
                    ? l10n.mockBridgeActive
                    : l10n.realBridgeActive,
              ),
              trailing: const Icon(Icons.chevron_right),
              onTap: () => context.go(AppRoutes.dev),
            ),
          ],
        ],
      ),
    );
  }

  static IconData _themeModeIcon(ThemeMode mode) => switch (mode) {
        ThemeMode.system => Icons.brightness_auto,
        ThemeMode.light => Icons.light_mode,
        ThemeMode.dark => Icons.dark_mode,
      };

  static String _themeModeLabel(AppLocalizations l10n, ThemeMode mode) => switch (mode) {
        ThemeMode.system => l10n.themeSystem,
        ThemeMode.light => l10n.themeLight,
        ThemeMode.dark => l10n.themeDark,
      };

  Future<void> _enableBiometrics(BuildContext context, WidgetRef ref) async {
    final l10n = AppLocalizations.of(context)!;
    final pinCtrl = TextEditingController();
    final pin = await showDialog<String>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(l10n.enterYourPin),
        content: TextField(
          controller: pinCtrl,
          obscureText: true,
          keyboardType: TextInputType.number,
          autofocus: true,
          decoration: InputDecoration(
            labelText: l10n.currentPinLabel,
            border: const OutlineInputBorder(),
          ),
          onSubmitted: (value) => Navigator.pop(ctx, value),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: Text(l10n.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, pinCtrl.text),
            child: Text(l10n.confirm),
          ),
        ],
      ),
    );

    if (pin == null || pin.isEmpty) return;

    // Verify the PIN is correct without disrupting auth state.
    try {
      await ref.read(authProvider.notifier).verifyPin(pin);
      await BiometricService.enable(pin);
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(l10n.biometricEnabled)),
        );
      }
    } catch (_) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(l10n.wrongPin)),
        );
      }
    }
  }

  void _showChangePinDialog(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final oldPinCtrl = TextEditingController();
    final newPinCtrl = TextEditingController();
    final confirmCtrl = TextEditingController();

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(l10n.changePinTitle),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: oldPinCtrl,
              obscureText: true,
              keyboardType: TextInputType.number,
              decoration: InputDecoration(
                labelText: l10n.currentPinLabel,
                border: const OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: newPinCtrl,
              obscureText: true,
              keyboardType: TextInputType.number,
              decoration: InputDecoration(
                labelText: l10n.newPinLabel,
                border: const OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: confirmCtrl,
              obscureText: true,
              keyboardType: TextInputType.number,
              decoration: InputDecoration(
                labelText: l10n.confirmNewPinLabel,
                border: const OutlineInputBorder(),
              ),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: Text(l10n.cancel),
          ),
          FilledButton(
            onPressed: () async {
              final oldPin = oldPinCtrl.text;
              final newPin = newPinCtrl.text;
              final confirm = confirmCtrl.text;

              if (newPin.length < 4) {
                if (ctx.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(content: Text(l10n.pinMinLength)),
                  );
                }
                return;
              }
              if (newPin != confirm) {
                if (ctx.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(content: Text(l10n.pinsDoNotMatch)),
                  );
                }
                return;
              }

              Navigator.pop(ctx);
              try {
                await ref.read(authProvider.notifier).changePin(oldPin, newPin);
                await BiometricService.updatePin(newPin);
                if (context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(content: Text(l10n.pinChangedSuccess)),
                  );
                }
              } catch (e) {
                if (context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(content: Text(l10n.pinChangeFailed(e.toString()))),
                  );
                }
              }
            },
            child: Text(l10n.change),
          ),
        ],
      ),
    );
  }

  void _showThemePicker(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final notifier = ref.read(themeModeProvider.notifier);
    final current = ref.read(themeModeProvider);

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(l10n.chooseTheme),
        contentPadding: EdgeInsets.zero,
        content: RadioGroup<ThemeMode>(
          groupValue: current,
          onChanged: (v) {
            if (v != null) notifier.setMode(v);
            Navigator.pop(ctx);
          },
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              for (final mode in ThemeMode.values)
                RadioListTile<ThemeMode>(
                  title: Text(_themeModeLabel(l10n, mode)),
                  secondary: Icon(_themeModeIcon(mode)),
                  value: mode,
                ),
            ],
          ),
        ),
      ),
    );
  }
}

class _SectionHeader extends StatelessWidget {
  final String title;
  const _SectionHeader(this.title);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 4),
      child: Text(
        title,
        style: Theme.of(context).textTheme.labelLarge?.copyWith(
              color: Theme.of(context).colorScheme.primary,
            ),
      ),
    );
  }
}

class _ProxyTile extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final proxy = ref.watch(proxyProvider);
    final hasProxy = proxy != null && proxy.isNotEmpty;

    return ListTile(
      leading: Icon(hasProxy ? Icons.vpn_lock : Icons.public),
      title: Text(l10n.proxy),
      subtitle: Text(hasProxy ? proxy : l10n.directConnection),
      trailing: const Icon(Icons.chevron_right),
      onTap: () => _showProxyDialog(context, ref, proxy),
    );
  }

  void _showProxyDialog(BuildContext context, WidgetRef ref, String? current) {
    final l10n = AppLocalizations.of(context)!;
    final controller = TextEditingController(text: current ?? '');

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(l10n.proxy),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              l10n.proxyDescription,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
            ),
            const SizedBox(height: 16),
            TextField(
              controller: controller,
              autofocus: true,
              decoration: InputDecoration(
                labelText: l10n.proxyUrlLabel,
                hintText: l10n.proxyUrlHint,
                border: const OutlineInputBorder(),
                prefixIcon: const Icon(Icons.link),
              ),
              onSubmitted: (_) {
                Navigator.pop(ctx);
                _apply(context, ref, controller.text);
              },
            ),
          ],
        ),
        actions: [
          if (current != null && current.isNotEmpty)
            TextButton(
              onPressed: () {
                Navigator.pop(ctx);
                _apply(context, ref, '');
              },
              child: Text(l10n.clear),
            ),
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: Text(l10n.cancel),
          ),
          FilledButton(
            onPressed: () {
              Navigator.pop(ctx);
              _apply(context, ref, controller.text);
            },
            child: Text(l10n.save),
          ),
        ],
      ),
    );
  }

  void _apply(BuildContext context, WidgetRef ref, String value) {
    final l10n = AppLocalizations.of(context)!;
    final url = value.trim().isEmpty ? null : value.trim();
    ref.read(proxyProvider.notifier).setProxy(url);
    if (context.mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(url != null ? l10n.proxySetTo(url) : l10n.proxyCleared),
        ),
      );
    }
  }
}
