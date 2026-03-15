import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_es.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
      : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations? of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations);
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
    delegate,
    GlobalMaterialLocalizations.delegate,
    GlobalCupertinoLocalizations.delegate,
    GlobalWidgetsLocalizations.delegate,
  ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('es')
  ];

  /// No description provided for @cancel.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get cancel;

  /// No description provided for @retry.
  ///
  /// In en, this message translates to:
  /// **'Retry'**
  String get retry;

  /// No description provided for @refresh.
  ///
  /// In en, this message translates to:
  /// **'Refresh'**
  String get refresh;

  /// No description provided for @confirm.
  ///
  /// In en, this message translates to:
  /// **'Confirm'**
  String get confirm;

  /// No description provided for @reject.
  ///
  /// In en, this message translates to:
  /// **'Reject'**
  String get reject;

  /// No description provided for @save.
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get save;

  /// No description provided for @clear.
  ///
  /// In en, this message translates to:
  /// **'Clear'**
  String get clear;

  /// No description provided for @change.
  ///
  /// In en, this message translates to:
  /// **'Change'**
  String get change;

  /// No description provided for @authenticate.
  ///
  /// In en, this message translates to:
  /// **'Authenticate'**
  String get authenticate;

  /// No description provided for @authenticating.
  ///
  /// In en, this message translates to:
  /// **'Authenticating...'**
  String get authenticating;

  /// No description provided for @unknownError.
  ///
  /// In en, this message translates to:
  /// **'Unknown error'**
  String get unknownError;

  /// No description provided for @invalidResponse.
  ///
  /// In en, this message translates to:
  /// **'Invalid response from server'**
  String get invalidResponse;

  /// No description provided for @errorWithMessage.
  ///
  /// In en, this message translates to:
  /// **'Error: {error}'**
  String errorWithMessage(String error);

  /// No description provided for @dismiss.
  ///
  /// In en, this message translates to:
  /// **'Dismiss'**
  String get dismiss;

  /// No description provided for @enable.
  ///
  /// In en, this message translates to:
  /// **'Enable'**
  String get enable;

  /// No description provided for @loading.
  ///
  /// In en, this message translates to:
  /// **'Loading...'**
  String get loading;

  /// No description provided for @navHome.
  ///
  /// In en, this message translates to:
  /// **'Home'**
  String get navHome;

  /// No description provided for @navPending.
  ///
  /// In en, this message translates to:
  /// **'Pending'**
  String get navPending;

  /// No description provided for @navHistory.
  ///
  /// In en, this message translates to:
  /// **'History'**
  String get navHistory;

  /// No description provided for @navSettings.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get navSettings;

  /// No description provided for @scanQr.
  ///
  /// In en, this message translates to:
  /// **'Scan QR'**
  String get scanQr;

  /// No description provided for @homeTitle.
  ///
  /// In en, this message translates to:
  /// **'Llave'**
  String get homeTitle;

  /// No description provided for @logOut.
  ///
  /// In en, this message translates to:
  /// **'Log out'**
  String get logOut;

  /// No description provided for @logOutTitle.
  ///
  /// In en, this message translates to:
  /// **'Log out?'**
  String get logOutTitle;

  /// No description provided for @logOutMessage.
  ///
  /// In en, this message translates to:
  /// **'This will clear your session. You can re-activate later.'**
  String get logOutMessage;

  /// No description provided for @noActiveSession.
  ///
  /// In en, this message translates to:
  /// **'No active session'**
  String get noActiveSession;

  /// No description provided for @activateToStart.
  ///
  /// In en, this message translates to:
  /// **'Activate your device to get started.'**
  String get activateToStart;

  /// No description provided for @activateDevice.
  ///
  /// In en, this message translates to:
  /// **'Activate Device'**
  String get activateDevice;

  /// No description provided for @qrAuthentication.
  ///
  /// In en, this message translates to:
  /// **'QR Authentication'**
  String get qrAuthentication;

  /// No description provided for @emailLabel.
  ///
  /// In en, this message translates to:
  /// **'Email'**
  String get emailLabel;

  /// No description provided for @phoneLabel.
  ///
  /// In en, this message translates to:
  /// **'Phone'**
  String get phoneLabel;

  /// No description provided for @levelLabel.
  ///
  /// In en, this message translates to:
  /// **'Level'**
  String get levelLabel;

  /// No description provided for @activeSince.
  ///
  /// In en, this message translates to:
  /// **'Active since {date}'**
  String activeSince(String date);

  /// No description provided for @registrationBasic.
  ///
  /// In en, this message translates to:
  /// **'Basic'**
  String get registrationBasic;

  /// No description provided for @registrationAdvanced.
  ///
  /// In en, this message translates to:
  /// **'Advanced'**
  String get registrationAdvanced;

  /// No description provided for @registrationSuperior.
  ///
  /// In en, this message translates to:
  /// **'Superior'**
  String get registrationSuperior;

  /// No description provided for @pendingTitle.
  ///
  /// In en, this message translates to:
  /// **'Pending Requests'**
  String get pendingTitle;

  /// No description provided for @pendingEmptyTitle.
  ///
  /// In en, this message translates to:
  /// **'No pending requests'**
  String get pendingEmptyTitle;

  /// No description provided for @pendingEmptySubtitle.
  ///
  /// In en, this message translates to:
  /// **'Authentication requests will appear here\nwhen detected by the server.'**
  String get pendingEmptySubtitle;

  /// No description provided for @requestConfirmed.
  ///
  /// In en, this message translates to:
  /// **'Request confirmed'**
  String get requestConfirmed;

  /// No description provided for @requestRejected.
  ///
  /// In en, this message translates to:
  /// **'Request rejected'**
  String get requestRejected;

  /// No description provided for @failed.
  ///
  /// In en, this message translates to:
  /// **'Failed'**
  String get failed;

  /// No description provided for @providerCode.
  ///
  /// In en, this message translates to:
  /// **'Provider code: {code}'**
  String providerCode(String code);

  /// No description provided for @actionRequestTitle.
  ///
  /// In en, this message translates to:
  /// **'{action} request?'**
  String actionRequestTitle(String action);

  /// No description provided for @actionRequestMessage.
  ///
  /// In en, this message translates to:
  /// **'Are you sure you want to {action} this authentication request?'**
  String actionRequestMessage(String action);

  /// No description provided for @historyTitle.
  ///
  /// In en, this message translates to:
  /// **'History'**
  String get historyTitle;

  /// No description provided for @historyEmptyTitle.
  ///
  /// In en, this message translates to:
  /// **'No operations found'**
  String get historyEmptyTitle;

  /// No description provided for @accepted.
  ///
  /// In en, this message translates to:
  /// **'Accepted'**
  String get accepted;

  /// No description provided for @rejected.
  ///
  /// In en, this message translates to:
  /// **'Rejected'**
  String get rejected;

  /// No description provided for @myDataTitle.
  ///
  /// In en, this message translates to:
  /// **'My Data'**
  String get myDataTitle;

  /// No description provided for @noData.
  ///
  /// In en, this message translates to:
  /// **'No data'**
  String get noData;

  /// No description provided for @accountInformation.
  ///
  /// In en, this message translates to:
  /// **'Account Information'**
  String get accountInformation;

  /// No description provided for @nifLabel.
  ///
  /// In en, this message translates to:
  /// **'NIF'**
  String get nifLabel;

  /// No description provided for @registrationLevel.
  ///
  /// In en, this message translates to:
  /// **'Registration Level'**
  String get registrationLevel;

  /// No description provided for @qrAuthTitle.
  ///
  /// In en, this message translates to:
  /// **'QR Authentication'**
  String get qrAuthTitle;

  /// No description provided for @useCamera.
  ///
  /// In en, this message translates to:
  /// **'Use camera'**
  String get useCamera;

  /// No description provided for @enterManually.
  ///
  /// In en, this message translates to:
  /// **'Enter manually'**
  String get enterManually;

  /// No description provided for @qrSuccess.
  ///
  /// In en, this message translates to:
  /// **'QR authentication successful'**
  String get qrSuccess;

  /// No description provided for @scanAnother.
  ///
  /// In en, this message translates to:
  /// **'Scan another'**
  String get scanAnother;

  /// No description provided for @pointCamera.
  ///
  /// In en, this message translates to:
  /// **'Point your camera at a Cl@ve QR code'**
  String get pointCamera;

  /// No description provided for @qrCodeAuthentication.
  ///
  /// In en, this message translates to:
  /// **'QR Code Authentication'**
  String get qrCodeAuthentication;

  /// No description provided for @pasteQrHint.
  ///
  /// In en, this message translates to:
  /// **'Paste the QR code value from a Cl@ve authentication request.'**
  String get pasteQrHint;

  /// No description provided for @qrCodeValueLabel.
  ///
  /// In en, this message translates to:
  /// **'QR code value'**
  String get qrCodeValueLabel;

  /// No description provided for @pasteQrContent.
  ///
  /// In en, this message translates to:
  /// **'Paste QR content here'**
  String get pasteQrContent;

  /// No description provided for @invalidPetitionData.
  ///
  /// In en, this message translates to:
  /// **'Invalid response: no petition data'**
  String get invalidPetitionData;

  /// No description provided for @confirmAuthTitle.
  ///
  /// In en, this message translates to:
  /// **'Confirm Authentication'**
  String get confirmAuthTitle;

  /// No description provided for @authRequestReceived.
  ///
  /// In en, this message translates to:
  /// **'An authentication request has been received:'**
  String get authRequestReceived;

  /// No description provided for @organizationLabel.
  ///
  /// In en, this message translates to:
  /// **'Organization'**
  String get organizationLabel;

  /// No description provided for @typeLabel.
  ///
  /// In en, this message translates to:
  /// **'Type'**
  String get typeLabel;

  /// No description provided for @dniAuthTitle.
  ///
  /// In en, this message translates to:
  /// **'DNI/NIE Authentication'**
  String get dniAuthTitle;

  /// No description provided for @deviceActivated.
  ///
  /// In en, this message translates to:
  /// **'Device activated successfully!'**
  String get deviceActivated;

  /// No description provided for @goToHome.
  ///
  /// In en, this message translates to:
  /// **'Go to Home'**
  String get goToHome;

  /// No description provided for @smsVerification.
  ///
  /// In en, this message translates to:
  /// **'SMS Verification'**
  String get smsVerification;

  /// No description provided for @smsSentTo.
  ///
  /// In en, this message translates to:
  /// **'An SMS verification code has been sent to:'**
  String get smsSentTo;

  /// No description provided for @smsCodeLabel.
  ///
  /// In en, this message translates to:
  /// **'SMS Code'**
  String get smsCodeLabel;

  /// No description provided for @smsCodeHint.
  ///
  /// In en, this message translates to:
  /// **'Enter the code from your SMS'**
  String get smsCodeHint;

  /// No description provided for @verifying.
  ///
  /// In en, this message translates to:
  /// **'Verifying...'**
  String get verifying;

  /// No description provided for @verifyActivate.
  ///
  /// In en, this message translates to:
  /// **'Verify & Activate'**
  String get verifyActivate;

  /// No description provided for @dniLogin.
  ///
  /// In en, this message translates to:
  /// **'DNI/NIE Login'**
  String get dniLogin;

  /// No description provided for @dniLoginDescription.
  ///
  /// In en, this message translates to:
  /// **'Authenticate using the data printed on your physical DNI/NIE card. No certificate or password needed.'**
  String get dniLoginDescription;

  /// No description provided for @nifDniNie.
  ///
  /// In en, this message translates to:
  /// **'NIF (DNI/NIE)'**
  String get nifDniNie;

  /// No description provided for @nifHint.
  ///
  /// In en, this message translates to:
  /// **'12345678A'**
  String get nifHint;

  /// No description provided for @nifRequired.
  ///
  /// In en, this message translates to:
  /// **'NIF is required'**
  String get nifRequired;

  /// No description provided for @nifMinLength.
  ///
  /// In en, this message translates to:
  /// **'NIF must be at least 8 characters'**
  String get nifMinLength;

  /// No description provided for @fechaValidezLabel.
  ///
  /// In en, this message translates to:
  /// **'Fecha de validez del DNI'**
  String get fechaValidezLabel;

  /// No description provided for @ddMmYyyy.
  ///
  /// In en, this message translates to:
  /// **'DD/MM/YYYY'**
  String get ddMmYyyy;

  /// No description provided for @fechaHelperText.
  ///
  /// In en, this message translates to:
  /// **'Expiry date printed on your DNI/NIE card'**
  String get fechaHelperText;

  /// No description provided for @dateRequired.
  ///
  /// In en, this message translates to:
  /// **'Date is required'**
  String get dateRequired;

  /// No description provided for @useDdMmYyyy.
  ///
  /// In en, this message translates to:
  /// **'Use DD/MM/YYYY format'**
  String get useDdMmYyyy;

  /// No description provided for @settingsTitle.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get settingsTitle;

  /// No description provided for @appearance.
  ///
  /// In en, this message translates to:
  /// **'Appearance'**
  String get appearance;

  /// No description provided for @theme.
  ///
  /// In en, this message translates to:
  /// **'Theme'**
  String get theme;

  /// No description provided for @security.
  ///
  /// In en, this message translates to:
  /// **'Security'**
  String get security;

  /// No description provided for @changePinTitle.
  ///
  /// In en, this message translates to:
  /// **'Change PIN'**
  String get changePinTitle;

  /// No description provided for @updatePinSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Update your unlock PIN'**
  String get updatePinSubtitle;

  /// No description provided for @biometricUnlock.
  ///
  /// In en, this message translates to:
  /// **'Biometric unlock'**
  String get biometricUnlock;

  /// No description provided for @biometricSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Use fingerprint or face to unlock'**
  String get biometricSubtitle;

  /// No description provided for @network.
  ///
  /// In en, this message translates to:
  /// **'Network'**
  String get network;

  /// No description provided for @device.
  ///
  /// In en, this message translates to:
  /// **'Device'**
  String get device;

  /// No description provided for @clearSession.
  ///
  /// In en, this message translates to:
  /// **'Clear Session'**
  String get clearSession;

  /// No description provided for @clearSessionSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Log out and clear saved credentials'**
  String get clearSessionSubtitle;

  /// No description provided for @sessionCleared.
  ///
  /// In en, this message translates to:
  /// **'Session cleared'**
  String get sessionCleared;

  /// No description provided for @deactivateDevice.
  ///
  /// In en, this message translates to:
  /// **'Deactivate Device'**
  String get deactivateDevice;

  /// No description provided for @deactivateSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Remove this device from Llave'**
  String get deactivateSubtitle;

  /// No description provided for @deactivateTitle.
  ///
  /// In en, this message translates to:
  /// **'Deactivate Device?'**
  String get deactivateTitle;

  /// No description provided for @deactivateMessage.
  ///
  /// In en, this message translates to:
  /// **'This will remove your device registration with Llave. You will need to activate again.'**
  String get deactivateMessage;

  /// No description provided for @deactivate.
  ///
  /// In en, this message translates to:
  /// **'Deactivate'**
  String get deactivate;

  /// No description provided for @about.
  ///
  /// In en, this message translates to:
  /// **'About'**
  String get about;

  /// No description provided for @version.
  ///
  /// In en, this message translates to:
  /// **'Version'**
  String get version;

  /// No description provided for @poweredBy.
  ///
  /// In en, this message translates to:
  /// **'Powered by'**
  String get poweredBy;

  /// No description provided for @poweredByValue.
  ///
  /// In en, this message translates to:
  /// **'Rust core (llave-core) via flutter_rust_bridge'**
  String get poweredByValue;

  /// No description provided for @developer.
  ///
  /// In en, this message translates to:
  /// **'Developer'**
  String get developer;

  /// No description provided for @developerTools.
  ///
  /// In en, this message translates to:
  /// **'Developer Tools'**
  String get developerTools;

  /// No description provided for @mockBridgeActive.
  ///
  /// In en, this message translates to:
  /// **'Mock bridge active'**
  String get mockBridgeActive;

  /// No description provided for @realBridgeActive.
  ///
  /// In en, this message translates to:
  /// **'Real bridge active'**
  String get realBridgeActive;

  /// No description provided for @themeSystem.
  ///
  /// In en, this message translates to:
  /// **'System default'**
  String get themeSystem;

  /// No description provided for @themeLight.
  ///
  /// In en, this message translates to:
  /// **'Light'**
  String get themeLight;

  /// No description provided for @themeDark.
  ///
  /// In en, this message translates to:
  /// **'Dark'**
  String get themeDark;

  /// No description provided for @enterYourPin.
  ///
  /// In en, this message translates to:
  /// **'Enter your PIN'**
  String get enterYourPin;

  /// No description provided for @currentPinLabel.
  ///
  /// In en, this message translates to:
  /// **'Current PIN'**
  String get currentPinLabel;

  /// No description provided for @biometricEnabled.
  ///
  /// In en, this message translates to:
  /// **'Biometric unlock enabled'**
  String get biometricEnabled;

  /// No description provided for @wrongPin.
  ///
  /// In en, this message translates to:
  /// **'Wrong PIN'**
  String get wrongPin;

  /// No description provided for @newPinLabel.
  ///
  /// In en, this message translates to:
  /// **'New PIN'**
  String get newPinLabel;

  /// No description provided for @confirmNewPinLabel.
  ///
  /// In en, this message translates to:
  /// **'Confirm New PIN'**
  String get confirmNewPinLabel;

  /// No description provided for @pinMinLength.
  ///
  /// In en, this message translates to:
  /// **'PIN must be at least 4 characters'**
  String get pinMinLength;

  /// No description provided for @pinsDoNotMatch.
  ///
  /// In en, this message translates to:
  /// **'New PINs do not match'**
  String get pinsDoNotMatch;

  /// No description provided for @pinChangedSuccess.
  ///
  /// In en, this message translates to:
  /// **'PIN changed successfully'**
  String get pinChangedSuccess;

  /// No description provided for @pinChangeFailed.
  ///
  /// In en, this message translates to:
  /// **'Failed to change PIN: {error}'**
  String pinChangeFailed(String error);

  /// No description provided for @chooseTheme.
  ///
  /// In en, this message translates to:
  /// **'Choose theme'**
  String get chooseTheme;

  /// No description provided for @proxy.
  ///
  /// In en, this message translates to:
  /// **'Proxy'**
  String get proxy;

  /// No description provided for @directConnection.
  ///
  /// In en, this message translates to:
  /// **'Direct connection'**
  String get directConnection;

  /// No description provided for @proxyDescription.
  ///
  /// In en, this message translates to:
  /// **'Route all AEAT requests through an HTTP, HTTPS, or SOCKS5 proxy. Useful when your IP has been rate-limited.'**
  String get proxyDescription;

  /// No description provided for @proxyUrlLabel.
  ///
  /// In en, this message translates to:
  /// **'Proxy URL'**
  String get proxyUrlLabel;

  /// No description provided for @proxyUrlHint.
  ///
  /// In en, this message translates to:
  /// **'socks5://127.0.0.1:1080'**
  String get proxyUrlHint;

  /// No description provided for @proxySetTo.
  ///
  /// In en, this message translates to:
  /// **'Proxy set to {url}'**
  String proxySetTo(String url);

  /// No description provided for @proxyCleared.
  ///
  /// In en, this message translates to:
  /// **'Proxy cleared'**
  String get proxyCleared;

  /// No description provided for @appName.
  ///
  /// In en, this message translates to:
  /// **'Llave'**
  String get appName;

  /// No description provided for @checkingSession.
  ///
  /// In en, this message translates to:
  /// **'Checking session...'**
  String get checkingSession;

  /// No description provided for @biometricFailed.
  ///
  /// In en, this message translates to:
  /// **'Biometric unlock failed. Enter your PIN.'**
  String get biometricFailed;

  /// No description provided for @wrongPinTryAgain.
  ///
  /// In en, this message translates to:
  /// **'Wrong PIN. Try again.'**
  String get wrongPinTryAgain;

  /// No description provided for @forgotPinTitle.
  ///
  /// In en, this message translates to:
  /// **'Forgot PIN?'**
  String get forgotPinTitle;

  /// No description provided for @forgotPinMessage.
  ///
  /// In en, this message translates to:
  /// **'This will erase your saved session. You will need to activate your device again.'**
  String get forgotPinMessage;

  /// No description provided for @eraseAndReset.
  ///
  /// In en, this message translates to:
  /// **'Erase & Reset'**
  String get eraseAndReset;

  /// No description provided for @enterPinToUnlock.
  ///
  /// In en, this message translates to:
  /// **'Enter your PIN to unlock'**
  String get enterPinToUnlock;

  /// No description provided for @pinLabel.
  ///
  /// In en, this message translates to:
  /// **'PIN'**
  String get pinLabel;

  /// No description provided for @unlock.
  ///
  /// In en, this message translates to:
  /// **'Unlock'**
  String get unlock;

  /// No description provided for @unlockWithBiometrics.
  ///
  /// In en, this message translates to:
  /// **'Unlock with biometrics'**
  String get unlockWithBiometrics;

  /// No description provided for @forgotPin.
  ///
  /// In en, this message translates to:
  /// **'Forgot PIN?'**
  String get forgotPin;

  /// No description provided for @activateLlave.
  ///
  /// In en, this message translates to:
  /// **'Activate Llave'**
  String get activateLlave;

  /// No description provided for @registerDevice.
  ///
  /// In en, this message translates to:
  /// **'Register this device with your Cl@ve account.'**
  String get registerDevice;

  /// No description provided for @activateWithDni.
  ///
  /// In en, this message translates to:
  /// **'Activate with DNI/NIE'**
  String get activateWithDni;

  /// No description provided for @dniActivateDescription.
  ///
  /// In en, this message translates to:
  /// **'You will need the data printed on your physical DNI/NIE card and access to the phone number registered with Cl@ve.'**
  String get dniActivateDescription;

  /// No description provided for @setYourPin.
  ///
  /// In en, this message translates to:
  /// **'Set Your PIN'**
  String get setYourPin;

  /// No description provided for @setPinDescription.
  ///
  /// In en, this message translates to:
  /// **'This PIN will be required every time you open the app. It protects your Llave credentials on this device.'**
  String get setPinDescription;

  /// No description provided for @confirmPinLabel.
  ///
  /// In en, this message translates to:
  /// **'Confirm PIN'**
  String get confirmPinLabel;

  /// No description provided for @setPin.
  ///
  /// In en, this message translates to:
  /// **'Set PIN'**
  String get setPin;

  /// No description provided for @pinTooShort.
  ///
  /// In en, this message translates to:
  /// **'PIN must be at least {length} characters'**
  String pinTooShort(int length);

  /// No description provided for @enableBiometricTitle.
  ///
  /// In en, this message translates to:
  /// **'Enable biometric unlock?'**
  String get enableBiometricTitle;

  /// No description provided for @enableBiometricMessage.
  ///
  /// In en, this message translates to:
  /// **'Use your fingerprint or face to unlock Llave instead of entering your PIN each time.'**
  String get enableBiometricMessage;

  /// No description provided for @notNow.
  ///
  /// In en, this message translates to:
  /// **'Not now'**
  String get notNow;

  /// No description provided for @ipBlockedTitle.
  ///
  /// In en, this message translates to:
  /// **'IP temporarily blocked by AEAT'**
  String get ipBlockedTitle;

  /// No description provided for @ipBlockedMessage.
  ///
  /// In en, this message translates to:
  /// **'Your IP address has exceeded the maximum number of failed attempts allowed per day. Try again tomorrow, or configure a proxy to use a different IP address.'**
  String get ipBlockedMessage;

  /// No description provided for @setProxy.
  ///
  /// In en, this message translates to:
  /// **'Set Proxy'**
  String get setProxy;

  /// No description provided for @biometricReason.
  ///
  /// In en, this message translates to:
  /// **'Unlock Llave with biometrics'**
  String get biometricReason;

  /// No description provided for @pinsDoNotMatchSetPin.
  ///
  /// In en, this message translates to:
  /// **'PINs do not match'**
  String get pinsDoNotMatchSetPin;

  /// No description provided for @unknownProvider.
  ///
  /// In en, this message translates to:
  /// **'Unknown provider'**
  String get unknownProvider;

  /// No description provided for @unknownType.
  ///
  /// In en, this message translates to:
  /// **'Unknown'**
  String get unknownType;

  /// No description provided for @typeDniNie.
  ///
  /// In en, this message translates to:
  /// **'DNI/NIE'**
  String get typeDniNie;

  /// No description provided for @typeQr.
  ///
  /// In en, this message translates to:
  /// **'QR'**
  String get typeQr;

  /// No description provided for @typeSms.
  ///
  /// In en, this message translates to:
  /// **'SMS'**
  String get typeSms;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en', 'es'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'es':
      return AppLocalizationsEs();
  }

  throw FlutterError(
      'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
      'an issue with the localizations generation tool. Please file an issue '
      'on GitHub with a reproducible sample app and the gen-l10n configuration '
      'that was used.');
}
