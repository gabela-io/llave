import 'dart:developer' as developer;
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'l10n/app_localizations.dart';
import 'package:logging/logging.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'src/rust/api/api.dart' as native_ffi;

import 'screens/home_screen.dart';
import 'screens/activate_screen.dart';
import 'screens/pending_screen.dart';
import 'screens/history_screen.dart';
import 'screens/settings_screen.dart';
import 'screens/splash_screen.dart';
import 'screens/unlock_screen.dart';
import 'screens/set_pin_screen.dart';
import 'screens/dev_screen.dart';
import 'screens/dni_auth_screen.dart';
import 'screens/mydata_screen.dart';
import 'screens/qr_auth_screen.dart';
import 'src/auth_provider.dart';
import 'src/routes.dart';
import 'src/theme_provider.dart';

Future<void> main() async {
  // Set up logging — forward all records to developer log (visible in
  // `flutter logs` / DevTools console).  In release mode only warnings+.
  Logger.root.level = kDebugMode ? Level.ALL : Level.WARNING;
  Logger.root.onRecord.listen((record) {
    developer.log(
      record.message,
      time: record.time,
      level: record.level.value,
      name: record.loggerName,
      error: record.error,
      stackTrace: record.stackTrace,
    );
  });

  WidgetsFlutterBinding.ensureInitialized();
  if (Platform.isLinux) {
    // Resolve the library path relative to the executable so dlopen can find it
    // regardless of RPATH/RUNPATH configuration.
    final exeDir = File(Platform.resolvedExecutable).parent.path;
    native_ffi.nativeLibPath = '$exeDir/lib/libllave_core_ffi.so';
  } else if (Platform.isAndroid) {
    native_ffi.nativeLibPath = 'libllave_core_ffi.so';
  }
  runApp(const ProviderScope(child: LlaveApp()));
}

/// Routes that require an active (decrypted) session.
const _protectedRoutes = {AppRoutes.home, AppRoutes.pending, AppRoutes.history, AppRoutes.settings, AppRoutes.qr};

/// Routes that should not be accessible when already authenticated.
const _guestOnlyRoutes = {AppRoutes.activate, AppRoutes.dniAuth};

/// Routes with their own navigation logic (never redirect away from them).
const _selfManagedRoutes = {AppRoutes.splash, AppRoutes.unlock, AppRoutes.setPin};

final _log = Logger('Router');

/// Riverpod provider for the app router.
///
/// Uses [refreshListenable] so go_router re-evaluates redirects when the auth
/// state changes — without recreating the entire router (which would reset
/// navigation to /splash and re-run init).
final routerProvider = Provider<GoRouter>((ref) {
  // Bridge between Riverpod and ChangeNotifier so go_router can listen.
  final notifier = _AuthChangeNotifier(ref);

  return GoRouter(
    initialLocation: AppRoutes.splash,
    refreshListenable: notifier,
    redirect: (context, state) {
      final auth = ref.read(authProvider);
      final location = state.uri.path;

      _log.fine('redirect: location=$location auth=${auth.runtimeType}');

      // Never redirect away from self-managed routes.
      if (_selfManagedRoutes.contains(location)) return null;

      final isAuthenticated = auth is AuthAuthenticated;
      final isLocked = auth is AuthLocked;
      final needsPin = auth is AuthNeedsPin;

      // Encrypted session on disk — force PIN entry.
      if (isLocked && location != AppRoutes.unlock) {
        _log.info('redirect: $location → ${AppRoutes.unlock} (session locked)');
        return AppRoutes.unlock;
      }

      // Just activated — force PIN setup.
      if (needsPin && location != AppRoutes.setPin) {
        _log.info('redirect: $location → ${AppRoutes.setPin} (needs PIN setup)');
        return AppRoutes.setPin;
      }

      // Redirect unauthenticated users away from protected routes.
      if (!isAuthenticated && _protectedRoutes.contains(location)) {
        _log.info('redirect: $location → ${AppRoutes.activate} (not authenticated)');
        return AppRoutes.activate;
      }

      // Redirect authenticated users away from guest-only routes.
      if (isAuthenticated && _guestOnlyRoutes.contains(location)) {
        _log.info('redirect: $location → ${AppRoutes.home} (already authenticated)');
        return AppRoutes.home;
      }

      return null;
    },
    routes: [
      GoRoute(path: AppRoutes.splash, builder: (_, __) => const SplashScreen()),
      GoRoute(path: AppRoutes.unlock, builder: (_, __) => const UnlockScreen()),
      GoRoute(path: AppRoutes.setPin, builder: (_, __) => const SetPinScreen()),
      GoRoute(path: AppRoutes.activate, builder: (_, __) => const ActivateScreen()),
      GoRoute(path: AppRoutes.dniAuth, builder: (_, __) => const DniAuthScreen()),
      ShellRoute(
        builder: (context, state, child) => AppShell(child: child),
        routes: [
          GoRoute(path: AppRoutes.home, builder: (_, __) => const HomeScreen()),
          GoRoute(path: AppRoutes.pending, builder: (_, __) => const PendingScreen()),
          GoRoute(path: AppRoutes.history, builder: (_, __) => const HistoryScreen()),
          GoRoute(path: AppRoutes.qr, builder: (_, __) => const QrAuthScreen()),
          GoRoute(path: AppRoutes.mydata, builder: (_, __) => const MyDataScreen()),
          GoRoute(
              path: AppRoutes.settings, builder: (_, __) => const SettingsScreen()),
          if (DevScreen.isEnabled)
            GoRoute(path: AppRoutes.dev, builder: (_, __) => const DevScreen()),
        ],
      ),
    ],
  );
});

/// Bridges Riverpod [authProvider] changes into a [ChangeNotifier] for
/// go_router's [refreshListenable].
class _AuthChangeNotifier extends ChangeNotifier {
  _AuthChangeNotifier(Ref ref) {
    ref.listen(authProvider, (_, __) => notifyListeners());
  }
}

class LlaveApp extends ConsumerWidget {
  const LlaveApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeMode = ref.watch(themeModeProvider);
    final router = ref.watch(routerProvider);

    return MaterialApp.router(
      onGenerateTitle: (context) => AppLocalizations.of(context)!.appName,
      debugShowCheckedModeBanner: false,
      themeMode: themeMode,
      theme: ThemeData(
        colorSchemeSeed: const Color(0xFF2563EB),
        useMaterial3: true,
        brightness: Brightness.light,
      ),
      darkTheme: ThemeData(
        colorSchemeSeed: const Color(0xFF2563EB),
        useMaterial3: true,
        brightness: Brightness.dark,
      ),
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      routerConfig: router,
    );
  }
}

/// Shell with bottom navigation bar for mobile, navigation rail for desktop.
class AppShell extends StatelessWidget {
  final Widget child;
  const AppShell({super.key, required this.child});

  static List<NavigationRailDestination> _railDestinations(AppLocalizations l10n) => [
    NavigationRailDestination(
        icon: const Icon(Icons.home_outlined),
        selectedIcon: const Icon(Icons.home),
        label: Text(l10n.navHome)),
    NavigationRailDestination(
        icon: const Icon(Icons.notifications_outlined),
        selectedIcon: const Icon(Icons.notifications),
        label: Text(l10n.navPending)),
    NavigationRailDestination(
        icon: const Icon(Icons.history_outlined),
        selectedIcon: const Icon(Icons.history),
        label: Text(l10n.navHistory)),
    NavigationRailDestination(
        icon: const Icon(Icons.settings_outlined),
        selectedIcon: const Icon(Icons.settings),
        label: Text(l10n.navSettings)),
  ];

  static List<NavigationDestination> _barDestinations(AppLocalizations l10n) => [
    NavigationDestination(
        icon: const Icon(Icons.home_outlined),
        selectedIcon: const Icon(Icons.home),
        label: l10n.navHome),
    NavigationDestination(
        icon: const Icon(Icons.notifications_outlined),
        selectedIcon: const Icon(Icons.notifications),
        label: l10n.navPending),
    NavigationDestination(
        icon: const Icon(Icons.history_outlined),
        selectedIcon: const Icon(Icons.history),
        label: l10n.navHistory),
    NavigationDestination(
        icon: const Icon(Icons.settings_outlined),
        selectedIcon: const Icon(Icons.settings),
        label: l10n.navSettings),
  ];

  static const _routes = [AppRoutes.home, AppRoutes.pending, AppRoutes.history, AppRoutes.settings];

  int _currentIndex(BuildContext context) {
    final location = GoRouterState.of(context).uri.toString();
    final idx = _routes.indexOf(location);
    return idx >= 0 ? idx : 0;
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final width = MediaQuery.sizeOf(context).width;
    final useRail = width >= 600;
    final selectedIndex = _currentIndex(context);
    final location = GoRouterState.of(context).uri.toString();
    final showFab = location != AppRoutes.settings;

    final fab = FloatingActionButton.extended(
      onPressed: () => context.go(AppRoutes.qr),
      tooltip: l10n.scanQr,
      icon: const Icon(Icons.qr_code_scanner),
      label: Text(l10n.scanQr),
    );

    if (useRail) {
      return Scaffold(
        floatingActionButton: showFab ? fab : null,
        body: Row(
          children: [
            NavigationRail(
              selectedIndex: selectedIndex,
              onDestinationSelected: (i) => context.go(_routes[i]),
              labelType: NavigationRailLabelType.all,
              destinations: _railDestinations(l10n),
            ),
            const VerticalDivider(thickness: 1, width: 1),
            Expanded(child: child),
          ],
        ),
      );
    }

    return Scaffold(
      floatingActionButton: showFab ? fab : null,
      body: child,
      bottomNavigationBar: NavigationBar(
        selectedIndex: selectedIndex,
        onDestinationSelected: (i) => context.go(_routes[i]),
        destinations: _barDestinations(l10n),
      ),
    );
  }
}
