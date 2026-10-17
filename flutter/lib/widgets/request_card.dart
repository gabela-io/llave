import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';

/// The app's security check for a Cl@ve Móvil request: who is asking, the
/// verification code to compare with the website, and "did you start this?".
/// Shared by the waiting and QR screens.
class RequestCard extends StatelessWidget {
  const RequestCard({
    super.key,
    required this.request,
    required this.busy,
    required this.onConfirm,
    required this.onReject,
  });

  final Map<String, dynamic> request;
  final bool busy;
  final VoidCallback onConfirm;
  final VoidCallback onReject;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);
    final organismo = request['organismo']?.toString() ?? l10n.unknownProvider;
    final code = request['codigoIdP']?.toString() ?? '';

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(l10n.requestFrom, style: theme.textTheme.labelLarge),
            Text(organismo, style: theme.textTheme.titleLarge),
            const SizedBox(height: 20),
            Text(l10n.verificationCodeLabel, style: theme.textTheme.labelLarge),
            Text(
              code,
              style: theme.textTheme.displaySmall?.copyWith(letterSpacing: 4),
            ),
            Text(
              l10n.verificationCodeHint,
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 20),
            Text(l10n.requestQuestion, style: theme.textTheme.bodyLarge),
            const SizedBox(height: 20),
            Row(
              children: [
                Expanded(
                  child: OutlinedButton(
                    onPressed: busy ? null : onReject,
                    style: OutlinedButton.styleFrom(
                      foregroundColor: theme.colorScheme.error,
                    ),
                    child: Text(l10n.noReject),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: FilledButton(
                    onPressed: busy ? null : onConfirm,
                    child: Text(l10n.yesConfirm),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
