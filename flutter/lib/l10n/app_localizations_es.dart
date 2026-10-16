// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Spanish Castilian (`es`).
class AppLocalizationsEs extends AppLocalizations {
  AppLocalizationsEs([String locale = 'es']) : super(locale);

  @override
  String get waitStart => 'Esperar un acceso';

  @override
  String get waitHint =>
      'Inicia un acceso con Cl@ve en una web y espera aquí la petición.';

  @override
  String get waitSearching => 'Esperando una petición de acceso…';

  @override
  String waitRemaining(int seconds) {
    return 'Quedan $seconds s';
  }

  @override
  String get waitTimeout =>
      'No ha llegado ninguna petición. Inicia el acceso en la web y vuelve a intentarlo.';

  @override
  String get requestFrom => 'Petición de acceso de';

  @override
  String get verificationCodeLabel => 'Código de verificación';

  @override
  String get verificationCodeHint =>
      'Debe coincidir con el código que muestra la web.';

  @override
  String get requestQuestion =>
      '¿Has iniciado tú este acceso, en la web oficial y no desde un enlace de un correo o SMS?';

  @override
  String get yesConfirm => 'Sí, confirmar';

  @override
  String get noReject => 'No, rechazar';

  @override
  String get pinCardTitle => 'Cl@ve PIN';

  @override
  String get pinCardHint =>
      'Solicita un PIN en la web de la AEAT y muéstralo aquí.';

  @override
  String get pinReveal => 'Mostrar PIN';

  @override
  String get pinHide => 'Ocultar';

  @override
  String get pinNoPending =>
      'No hay ningún PIN pendiente. Solicítalo primero en la web de la AEAT.';

  @override
  String get pinBiometricReason => 'Confirma para ver tu Cl@ve PIN';

  @override
  String pinValidFor(int seconds) {
    return 'Válido durante $seconds s';
  }

  @override
  String get cancel => 'Cancelar';

  @override
  String get retry => 'Reintentar';

  @override
  String get refresh => 'Actualizar';

  @override
  String get confirm => 'Confirmar';

  @override
  String get save => 'Guardar';

  @override
  String get clear => 'Borrar';

  @override
  String get change => 'Cambiar';

  @override
  String get authenticate => 'Autenticar';

  @override
  String get authenticating => 'Autenticando...';

  @override
  String get unknownError => 'Error desconocido';

  @override
  String get errorNoConnection =>
      'No se ha podido conectar con los servidores de la AEAT. Comprueba tu conexión e inténtalo de nuevo.';

  @override
  String get errorNoSession =>
      'Este dispositivo no tiene sesión de Cl@ve. Vuelve a activarlo.';

  @override
  String get errorDeviceNotActive =>
      'Este dispositivo ya no está activo en Cl@ve. Vuelve a activarlo.';

  @override
  String get errorNotRegistered =>
      'Este NIF no está registrado en Cl@ve. Regístrate primero en la web de la AEAT o en una oficina.';

  @override
  String get errorNoRegisteredPhone =>
      'Tu registro en Cl@ve no tiene teléfono móvil. Añádelo primero en la web de la AEAT.';

  @override
  String get errorAppOutdated =>
      'Los servidores de la AEAT ya no aceptan esta versión de llave. Actualiza la app.';

  @override
  String get errorInvalidQr =>
      'Este no es un código QR de Cl@ve. Escanea el código que muestra la web de la AEAT.';

  @override
  String get errorDniData =>
      'Los servidores de la AEAT no han aceptado los datos del documento. Revisa el NIF y la fecha o el número de soporte.';

  @override
  String get errorInvalidNif => 'El NIF no es válido.';

  @override
  String get errorUnexpectedResponse =>
      'Respuesta inesperada de los servidores de la AEAT. Inténtalo más tarde.';

  @override
  String get errorUnexpected => 'Algo ha fallado. Inténtalo de nuevo.';

  @override
  String get invalidResponse => 'Respuesta no válida del servidor';

  @override
  String errorWithMessage(String error) {
    return 'Error: $error';
  }

  @override
  String get dismiss => 'Cerrar';

  @override
  String get enable => 'Activar';

  @override
  String get loading => 'Cargando...';

  @override
  String get navHome => 'Inicio';

  @override
  String get navPending => 'Pendientes';

  @override
  String get navHistory => 'Historial';

  @override
  String get navSettings => 'Ajustes';

  @override
  String get scanQr => 'Escanear QR';

  @override
  String get homeTitle => 'Llave';

  @override
  String get logOut => 'Cerrar sesión';

  @override
  String get logOutTitle => '¿Cerrar sesión?';

  @override
  String get logOutMessage =>
      'Se borrará tu sesión. Podrás reactivarla más tarde.';

  @override
  String get qrAuthentication => 'Autenticación QR';

  @override
  String get emailLabel => 'Correo electrónico';

  @override
  String get phoneLabel => 'Teléfono';

  @override
  String get levelLabel => 'Nivel';

  @override
  String activeSince(String date) {
    return 'Activo desde $date';
  }

  @override
  String get registrationBasic => 'Básico';

  @override
  String get registrationAdvanced => 'Avanzado';

  @override
  String get registrationSuperior => 'Superior';

  @override
  String get pendingTitle => 'Solicitudes pendientes';

  @override
  String get requestConfirmed => 'Solicitud confirmada';

  @override
  String get requestRejected => 'Solicitud rechazada';

  @override
  String get historyTitle => 'Historial';

  @override
  String get historyEmptyTitle => 'No se encontraron operaciones';

  @override
  String get accepted => 'Aceptada';

  @override
  String get rejected => 'Rechazada';

  @override
  String get qrAuthTitle => 'Autenticación QR';

  @override
  String get useCamera => 'Usar cámara';

  @override
  String get enterManually => 'Introducir manualmente';

  @override
  String get qrSuccess => 'Autenticación QR exitosa';

  @override
  String get scanAnother => 'Escanear otro';

  @override
  String get pointCamera => 'Apunta tu cámara al código QR de Cl@ve';

  @override
  String get qrCodeAuthentication => 'Autenticación con código QR';

  @override
  String get pasteQrHint =>
      'Pega el valor del código QR de una solicitud de autenticación de Cl@ve.';

  @override
  String get qrCodeValueLabel => 'Valor del código QR';

  @override
  String get pasteQrContent => 'Pega el contenido del QR aquí';

  @override
  String get invalidPetitionData =>
      'Respuesta no válida: sin datos de petición';

  @override
  String get dniAuthTitle => 'Autenticación DNI/NIE';

  @override
  String get smsVerification => 'Verificación por SMS';

  @override
  String get smsSentTo => 'Se ha enviado un código de verificación por SMS a:';

  @override
  String get smsCodeLabel => 'Código SMS';

  @override
  String get smsCodeHint => 'Introduce el código de tu SMS';

  @override
  String get verifying => 'Verificando...';

  @override
  String get verifyActivate => 'Verificar y activar';

  @override
  String get dniLogin => 'Acceso con DNI/NIE';

  @override
  String get dniLoginDescription =>
      'Autentícate con los datos impresos en tu DNI/NIE físico. No se necesita certificado ni contraseña.';

  @override
  String get nifDniNie => 'NIF (DNI/NIE)';

  @override
  String get nifHint => '12345678A';

  @override
  String get nifRequired => 'El NIF es obligatorio';

  @override
  String get nifMinLength => 'El NIF debe tener al menos 8 caracteres';

  @override
  String get fechaValidezLabel => 'Fecha de validez del DNI';

  @override
  String get ddMmYyyy => 'DD-MM-AAAA';

  @override
  String get fechaHelperText => 'Fecha de caducidad impresa en tu DNI';

  @override
  String get dateRequired => 'La fecha es obligatoria';

  @override
  String get useDdMmYyyy => 'Usa el formato DD-MM-AAAA';

  @override
  String get birthDateLabel => 'Fecha de nacimiento';

  @override
  String get birthDateHelperText => 'La que figura en tu documento';

  @override
  String get supportNumberLabel => 'Número de soporte';

  @override
  String get supportNumberHelperText =>
      'Impreso en tu tarjeta NIE, p. ej. E12345678';

  @override
  String get supportNumberRequired => 'El número de soporte es obligatorio';

  @override
  String get birthDateChallenge =>
      'Por seguridad, introduce también tu fecha de nacimiento.';

  @override
  String get settingsTitle => 'Ajustes';

  @override
  String get appearance => 'Apariencia';

  @override
  String get theme => 'Tema';

  @override
  String get security => 'Seguridad';

  @override
  String get changePinTitle => 'Cambiar PIN';

  @override
  String get updatePinSubtitle => 'Actualiza tu PIN de desbloqueo';

  @override
  String get biometricUnlock => 'Desbloqueo biométrico';

  @override
  String get biometricSubtitle =>
      'Usa huella dactilar o rostro para desbloquear';

  @override
  String get network => 'Red';

  @override
  String get device => 'Dispositivo';

  @override
  String get clearSession => 'Borrar sesión';

  @override
  String get clearSessionSubtitle =>
      'Cerrar sesión y borrar credenciales guardadas';

  @override
  String get sessionCleared => 'Sesión borrada';

  @override
  String get deactivateDevice => 'Desactivar dispositivo';

  @override
  String get deactivateSubtitle => 'Eliminar este dispositivo de Llave';

  @override
  String get deactivateTitle => '¿Desactivar dispositivo?';

  @override
  String get deactivateMessage =>
      'Esto eliminará el registro de tu dispositivo en Llave. Tendrás que activarlo de nuevo.';

  @override
  String get deactivate => 'Desactivar';

  @override
  String get about => 'Acerca de';

  @override
  String get version => 'Versión';

  @override
  String get poweredBy => 'Desarrollado con';

  @override
  String get poweredByValue => 'Núcleo Rust (llave-core) vía FFI';

  @override
  String get developer => 'Desarrollador';

  @override
  String get developerTools => 'Herramientas de desarrollo';

  @override
  String get mockBridgeActive => 'Bridge simulado activo';

  @override
  String get realBridgeActive => 'Bridge real activo';

  @override
  String get themeSystem => 'Predeterminado del sistema';

  @override
  String get themeLight => 'Claro';

  @override
  String get themeDark => 'Oscuro';

  @override
  String get enterYourPin => 'Introduce tu PIN';

  @override
  String get currentPinLabel => 'PIN actual';

  @override
  String get biometricEnabled => 'Desbloqueo biométrico activado';

  @override
  String get wrongPin => 'PIN incorrecto';

  @override
  String get newPinLabel => 'Nuevo PIN';

  @override
  String get confirmNewPinLabel => 'Confirmar nuevo PIN';

  @override
  String get pinMinLength => 'El PIN debe tener al menos 4 caracteres';

  @override
  String get pinsDoNotMatch => 'Los nuevos PINs no coinciden';

  @override
  String get pinChangedSuccess => 'PIN cambiado correctamente';

  @override
  String pinChangeFailed(String error) {
    return 'Error al cambiar el PIN: $error';
  }

  @override
  String get chooseTheme => 'Elegir tema';

  @override
  String get proxy => 'Proxy';

  @override
  String get directConnection => 'Conexión directa';

  @override
  String get proxyDescription =>
      'Enruta todas las solicitudes a la AEAT a través de un proxy HTTP, HTTPS o SOCKS5. Útil cuando tu IP ha sido limitada.';

  @override
  String get proxyUrlLabel => 'URL del proxy';

  @override
  String get proxyUrlHint => 'socks5://127.0.0.1:1080';

  @override
  String proxySetTo(String url) {
    return 'Proxy configurado: $url';
  }

  @override
  String get proxyCleared => 'Proxy borrado';

  @override
  String get appName => 'Llave';

  @override
  String get checkingSession => 'Comprobando sesión...';

  @override
  String get biometricFailed =>
      'Desbloqueo biométrico fallido. Introduce tu PIN.';

  @override
  String get wrongPinTryAgain => 'PIN incorrecto. Inténtalo de nuevo.';

  @override
  String get forgotPinTitle => '¿Olvidaste el PIN?';

  @override
  String get forgotPinMessage =>
      'Esto borrará tu sesión guardada. Tendrás que activar tu dispositivo de nuevo.';

  @override
  String get eraseAndReset => 'Borrar y restablecer';

  @override
  String get enterPinToUnlock => 'Introduce tu PIN para desbloquear';

  @override
  String get pinLabel => 'PIN';

  @override
  String get unlock => 'Desbloquear';

  @override
  String get unlockWithBiometrics => 'Desbloquear con biometría';

  @override
  String get forgotPin => '¿Olvidaste el PIN?';

  @override
  String get setYourPin => 'Establece tu PIN';

  @override
  String get setPinDescription =>
      'Este PIN se solicitará cada vez que abras la aplicación. Protege tus credenciales de Llave en este dispositivo.';

  @override
  String get confirmPinLabel => 'Confirmar PIN';

  @override
  String get setPin => 'Establecer PIN';

  @override
  String pinTooShort(int length) {
    return 'El PIN debe tener al menos $length caracteres';
  }

  @override
  String get enableBiometricTitle => '¿Activar desbloqueo biométrico?';

  @override
  String get enableBiometricMessage =>
      'Usa tu huella dactilar o rostro para desbloquear Llave en lugar de introducir tu PIN cada vez.';

  @override
  String get notNow => 'Ahora no';

  @override
  String get ipBlockedTitle => 'IP bloqueada temporalmente por la AEAT';

  @override
  String get ipBlockedMessage =>
      'Tu dirección IP ha superado el número máximo de intentos fallidos permitidos por día. Inténtalo mañana, o configura un proxy para usar una dirección IP diferente.';

  @override
  String get setProxy => 'Configurar proxy';

  @override
  String get biometricReason => 'Desbloquear Llave con biometría';

  @override
  String get pinsDoNotMatchSetPin => 'Los PINs no coinciden';

  @override
  String get unknownProvider => 'Proveedor desconocido';

  @override
  String get unknownType => 'Desconocido';

  @override
  String get typeDniNie => 'DNI/NIE';

  @override
  String get typeQr => 'QR';

  @override
  String get typeSms => 'SMS';
}
