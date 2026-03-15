import 'package:flutter/material.dart';
import '../l10n/app_localizations.dart';
import 'package:go_router/go_router.dart';

import 'routes.dart';

/// Sentinel prefix emitted by Rust `LlaveError::IpRateLimited`.
const _prefix = 'IP rate-limited by AEAT';

/// Returns `true` when an error string indicates the AEAT server has
/// temporarily blocked this IP address (error code 16600).
bool isIpRateLimited(String? error) {
  if (error == null) return false;
  return error.contains(_prefix);
}

/// Shows a persistent [MaterialBanner] warning the user that their IP
/// has been temporarily blocked by AEAT.
///
/// The banner stays visible until the user dismisses it.  Includes a
/// "Set Proxy" action that navigates to Settings.  Safe to call multiple
/// times — clears previous banners first to avoid stacking.
void showIpRateLimitBanner(BuildContext context) {
  final l10n = AppLocalizations.of(context)!;
  final theme = Theme.of(context);
  final messenger = ScaffoldMessenger.of(context);

  // Avoid stacking multiple banners.
  messenger.clearMaterialBanners();

  messenger.showMaterialBanner(
    MaterialBanner(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 12),
      leading: Icon(Icons.block, color: theme.colorScheme.error),
      backgroundColor: theme.colorScheme.errorContainer,
      content: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            l10n.ipBlockedTitle,
            style: theme.textTheme.titleSmall?.copyWith(
              color: theme.colorScheme.onErrorContainer,
              fontWeight: FontWeight.bold,
            ),
          ),
          const SizedBox(height: 4),
          Text(
            l10n.ipBlockedMessage,
            style: theme.textTheme.bodySmall?.copyWith(
              color: theme.colorScheme.onErrorContainer,
            ),
          ),
        ],
      ),
      actions: [
        TextButton(
          onPressed: () {
            messenger.hideCurrentMaterialBanner();
            context.go(AppRoutes.settings);
          },
          child: Text(
            l10n.setProxy,
            style: TextStyle(color: theme.colorScheme.onErrorContainer),
          ),
        ),
        TextButton(
          onPressed: () => messenger.hideCurrentMaterialBanner(),
          child: Text(
            l10n.dismiss,
            style: TextStyle(color: theme.colorScheme.onErrorContainer),
          ),
        ),
      ],
    ),
  );
}
