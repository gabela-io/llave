// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get waitStart => 'Wait for a login';

  @override
  String get waitHint =>
      'Start a Cl@ve login on a website, then wait here for the request.';

  @override
  String get waitSearching => 'Waiting for a login request…';

  @override
  String waitRemaining(int seconds) {
    return '$seconds s left';
  }

  @override
  String get waitTimeout =>
      'No request arrived. Start the login on the website and try again.';

  @override
  String get requestFrom => 'Login request from';

  @override
  String get verificationCodeLabel => 'Verification code';

  @override
  String get verificationCodeHint =>
      'It must match the code shown on the website.';

  @override
  String get requestQuestion =>
      'Did you start this login yourself, on the official website, and not from a link in an email or SMS?';

  @override
  String get yesConfirm => 'Yes, confirm';

  @override
  String get noReject => 'No, reject';

  @override
  String get pinCardTitle => 'Cl@ve PIN';

  @override
  String get pinCardHint =>
      'Request a PIN on the AEAT website, then reveal it here.';

  @override
  String get pinReveal => 'Show PIN';

  @override
  String get pinHide => 'Hide';

  @override
  String get pinNoPending =>
      'There is no pending PIN. Request one on the AEAT website first.';

  @override
  String get pinBiometricReason => 'Confirm to show your Cl@ve PIN';

  @override
  String pinValidFor(int seconds) {
    return 'Valid for $seconds s';
  }

  @override
  String get cancel => 'Cancel';

  @override
  String get retry => 'Retry';

  @override
  String get refresh => 'Refresh';

  @override
  String get confirm => 'Confirm';

  @override
  String get reject => 'Reject';

  @override
  String get save => 'Save';

  @override
  String get clear => 'Clear';

  @override
  String get change => 'Change';

  @override
  String get authenticate => 'Authenticate';

  @override
  String get authenticating => 'Authenticating...';

  @override
  String get unknownError => 'Unknown error';

  @override
  String get errorNoConnection =>
      'Could not reach the AEAT servers. Check your connection and try again.';

  @override
  String get errorNoSession =>
      'This device has no Cl@ve session. Activate it again.';

  @override
  String get errorDeviceNotActive =>
      'This device is no longer active in Cl@ve. Activate it again.';

  @override
  String get errorNotRegistered =>
      'This NIF is not registered in Cl@ve. Register on the AEAT website or at an office first.';

  @override
  String get errorNoRegisteredPhone =>
      'Your Cl@ve registration has no mobile phone. Add one on the AEAT website first.';

  @override
  String get errorAppOutdated =>
      'The AEAT servers no longer accept this version of llave. Update the app.';

  @override
  String get errorInvalidQr =>
      'This is not a Cl@ve QR code. Scan the code shown on the AEAT website.';

  @override
  String get errorDniData =>
      'The AEAT servers did not accept the document data. Check the NIF and the date or support number.';

  @override
  String get errorInvalidNif => 'The NIF is not valid.';

  @override
  String get errorUnexpectedResponse =>
      'Unexpected response from the AEAT servers. Try again later.';

  @override
  String get invalidResponse => 'Invalid response from server';

  @override
  String errorWithMessage(String error) {
    return 'Error: $error';
  }

  @override
  String get dismiss => 'Dismiss';

  @override
  String get enable => 'Enable';

  @override
  String get loading => 'Loading...';

  @override
  String get navHome => 'Home';

  @override
  String get navPending => 'Pending';

  @override
  String get navHistory => 'History';

  @override
  String get navSettings => 'Settings';

  @override
  String get scanQr => 'Scan QR';

  @override
  String get homeTitle => 'Llave';

  @override
  String get logOut => 'Log out';

  @override
  String get logOutTitle => 'Log out?';

  @override
  String get logOutMessage =>
      'This will clear your session. You can re-activate later.';

  @override
  String get noActiveSession => 'No active session';

  @override
  String get activateToStart => 'Activate your device to get started.';

  @override
  String get activateDevice => 'Activate Device';

  @override
  String get qrAuthentication => 'QR Authentication';

  @override
  String get emailLabel => 'Email';

  @override
  String get phoneLabel => 'Phone';

  @override
  String get levelLabel => 'Level';

  @override
  String activeSince(String date) {
    return 'Active since $date';
  }

  @override
  String get registrationBasic => 'Basic';

  @override
  String get registrationAdvanced => 'Advanced';

  @override
  String get registrationSuperior => 'Superior';

  @override
  String get pendingTitle => 'Pending Requests';

  @override
  String get pendingEmptyTitle => 'No pending requests';

  @override
  String get pendingEmptySubtitle =>
      'Authentication requests will appear here\nwhen detected by the server.';

  @override
  String get requestConfirmed => 'Request confirmed';

  @override
  String get requestRejected => 'Request rejected';

  @override
  String get failed => 'Failed';

  @override
  String providerCode(String code) {
    return 'Provider code: $code';
  }

  @override
  String actionRequestTitle(String action) {
    return '$action request?';
  }

  @override
  String actionRequestMessage(String action) {
    return 'Are you sure you want to $action this authentication request?';
  }

  @override
  String get historyTitle => 'History';

  @override
  String get historyEmptyTitle => 'No operations found';

  @override
  String get accepted => 'Accepted';

  @override
  String get rejected => 'Rejected';

  @override
  String get myDataTitle => 'My Data';

  @override
  String get noData => 'No data';

  @override
  String get accountInformation => 'Account Information';

  @override
  String get nifLabel => 'NIF';

  @override
  String get registrationLevel => 'Registration Level';

  @override
  String get qrAuthTitle => 'QR Authentication';

  @override
  String get useCamera => 'Use camera';

  @override
  String get enterManually => 'Enter manually';

  @override
  String get qrSuccess => 'QR authentication successful';

  @override
  String get scanAnother => 'Scan another';

  @override
  String get pointCamera => 'Point your camera at a Cl@ve QR code';

  @override
  String get qrCodeAuthentication => 'QR Code Authentication';

  @override
  String get pasteQrHint =>
      'Paste the QR code value from a Cl@ve authentication request.';

  @override
  String get qrCodeValueLabel => 'QR code value';

  @override
  String get pasteQrContent => 'Paste QR content here';

  @override
  String get invalidPetitionData => 'Invalid response: no petition data';

  @override
  String get dniAuthTitle => 'DNI/NIE Authentication';

  @override
  String get deviceActivated => 'Device activated successfully!';

  @override
  String get goToHome => 'Go to Home';

  @override
  String get smsVerification => 'SMS Verification';

  @override
  String get smsSentTo => 'An SMS verification code has been sent to:';

  @override
  String get smsCodeLabel => 'SMS Code';

  @override
  String get smsCodeHint => 'Enter the code from your SMS';

  @override
  String get verifying => 'Verifying...';

  @override
  String get verifyActivate => 'Verify & Activate';

  @override
  String get dniLogin => 'DNI/NIE Login';

  @override
  String get dniLoginDescription =>
      'Authenticate using the data printed on your physical DNI/NIE card. No certificate or password needed.';

  @override
  String get nifDniNie => 'NIF (DNI/NIE)';

  @override
  String get nifHint => '12345678A';

  @override
  String get nifRequired => 'NIF is required';

  @override
  String get nifMinLength => 'NIF must be at least 8 characters';

  @override
  String get fechaValidezLabel => 'ID card validity date';

  @override
  String get ddMmYyyy => 'DD-MM-YYYY';

  @override
  String get fechaHelperText => 'Expiry date printed on your DNI';

  @override
  String get dateRequired => 'Date is required';

  @override
  String get useDdMmYyyy => 'Use DD-MM-YYYY format';

  @override
  String get birthDateLabel => 'Date of birth';

  @override
  String get birthDateHelperText => 'As shown on your document';

  @override
  String get supportNumberLabel => 'Support number';

  @override
  String get supportNumberHelperText =>
      'Printed on your NIE card, e.g. E12345678';

  @override
  String get supportNumberRequired => 'Support number is required';

  @override
  String get birthDateChallenge =>
      'For extra security, also enter your date of birth.';

  @override
  String get settingsTitle => 'Settings';

  @override
  String get appearance => 'Appearance';

  @override
  String get theme => 'Theme';

  @override
  String get security => 'Security';

  @override
  String get changePinTitle => 'Change PIN';

  @override
  String get updatePinSubtitle => 'Update your unlock PIN';

  @override
  String get biometricUnlock => 'Biometric unlock';

  @override
  String get biometricSubtitle => 'Use fingerprint or face to unlock';

  @override
  String get network => 'Network';

  @override
  String get device => 'Device';

  @override
  String get clearSession => 'Clear Session';

  @override
  String get clearSessionSubtitle => 'Log out and clear saved credentials';

  @override
  String get sessionCleared => 'Session cleared';

  @override
  String get deactivateDevice => 'Deactivate Device';

  @override
  String get deactivateSubtitle => 'Remove this device from Llave';

  @override
  String get deactivateTitle => 'Deactivate Device?';

  @override
  String get deactivateMessage =>
      'This will remove your device registration with Llave. You will need to activate again.';

  @override
  String get deactivate => 'Deactivate';

  @override
  String get about => 'About';

  @override
  String get version => 'Version';

  @override
  String get poweredBy => 'Powered by';

  @override
  String get poweredByValue => 'Rust core (llave-core) via flutter_rust_bridge';

  @override
  String get developer => 'Developer';

  @override
  String get developerTools => 'Developer Tools';

  @override
  String get mockBridgeActive => 'Mock bridge active';

  @override
  String get realBridgeActive => 'Real bridge active';

  @override
  String get themeSystem => 'System default';

  @override
  String get themeLight => 'Light';

  @override
  String get themeDark => 'Dark';

  @override
  String get enterYourPin => 'Enter your PIN';

  @override
  String get currentPinLabel => 'Current PIN';

  @override
  String get biometricEnabled => 'Biometric unlock enabled';

  @override
  String get wrongPin => 'Wrong PIN';

  @override
  String get newPinLabel => 'New PIN';

  @override
  String get confirmNewPinLabel => 'Confirm New PIN';

  @override
  String get pinMinLength => 'PIN must be at least 4 characters';

  @override
  String get pinsDoNotMatch => 'New PINs do not match';

  @override
  String get pinChangedSuccess => 'PIN changed successfully';

  @override
  String pinChangeFailed(String error) {
    return 'Failed to change PIN: $error';
  }

  @override
  String get chooseTheme => 'Choose theme';

  @override
  String get proxy => 'Proxy';

  @override
  String get directConnection => 'Direct connection';

  @override
  String get proxyDescription =>
      'Route all AEAT requests through an HTTP, HTTPS, or SOCKS5 proxy. Useful when your IP has been rate-limited.';

  @override
  String get proxyUrlLabel => 'Proxy URL';

  @override
  String get proxyUrlHint => 'socks5://127.0.0.1:1080';

  @override
  String proxySetTo(String url) {
    return 'Proxy set to $url';
  }

  @override
  String get proxyCleared => 'Proxy cleared';

  @override
  String get appName => 'Llave';

  @override
  String get checkingSession => 'Checking session...';

  @override
  String get biometricFailed => 'Biometric unlock failed. Enter your PIN.';

  @override
  String get wrongPinTryAgain => 'Wrong PIN. Try again.';

  @override
  String get forgotPinTitle => 'Forgot PIN?';

  @override
  String get forgotPinMessage =>
      'This will erase your saved session. You will need to activate your device again.';

  @override
  String get eraseAndReset => 'Erase & Reset';

  @override
  String get enterPinToUnlock => 'Enter your PIN to unlock';

  @override
  String get pinLabel => 'PIN';

  @override
  String get unlock => 'Unlock';

  @override
  String get unlockWithBiometrics => 'Unlock with biometrics';

  @override
  String get forgotPin => 'Forgot PIN?';

  @override
  String get activateLlave => 'Activate Llave';

  @override
  String get registerDevice => 'Register this device with your Cl@ve account.';

  @override
  String get activateWithDni => 'Activate with DNI/NIE';

  @override
  String get dniActivateDescription =>
      'You will need the data printed on your physical DNI/NIE card and access to the phone number registered with Cl@ve.';

  @override
  String get setYourPin => 'Set Your PIN';

  @override
  String get setPinDescription =>
      'This PIN will be required every time you open the app. It protects your Llave credentials on this device.';

  @override
  String get confirmPinLabel => 'Confirm PIN';

  @override
  String get setPin => 'Set PIN';

  @override
  String pinTooShort(int length) {
    return 'PIN must be at least $length characters';
  }

  @override
  String get enableBiometricTitle => 'Enable biometric unlock?';

  @override
  String get enableBiometricMessage =>
      'Use your fingerprint or face to unlock Llave instead of entering your PIN each time.';

  @override
  String get notNow => 'Not now';

  @override
  String get ipBlockedTitle => 'IP temporarily blocked by AEAT';

  @override
  String get ipBlockedMessage =>
      'Your IP address has exceeded the maximum number of failed attempts allowed per day. Try again tomorrow, or configure a proxy to use a different IP address.';

  @override
  String get setProxy => 'Set Proxy';

  @override
  String get biometricReason => 'Unlock Llave with biometrics';

  @override
  String get pinsDoNotMatchSetPin => 'PINs do not match';

  @override
  String get unknownProvider => 'Unknown provider';

  @override
  String get unknownType => 'Unknown';

  @override
  String get typeDniNie => 'DNI/NIE';

  @override
  String get typeQr => 'QR';

  @override
  String get typeSms => 'SMS';
}
