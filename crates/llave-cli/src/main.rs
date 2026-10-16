use clap::{Parser, Subcommand};
use serde_json::json;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use llave_core::config::validate_nif;
use llave_core::error::{LlaveError, Result};
use llave_core::{
    init_storage, Config, KeyringStorage, LlaveClient, PinRequest, SealedFileStorage, Session,
};

#[derive(Parser)]
#[command(
    name = "llave",
    about = "CLI tool for Llave authentication (Spain's digital identity system)",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output plain text instead of JSON
    #[arg(long, global = true)]
    plain: bool,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Proxy URL (http, https, or socks5). Overrides config file.
    /// Example: socks5://127.0.0.1:1080
    #[arg(long, global = true, env = "LLAVE_PROXY")]
    proxy: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Activate this device with Llave (DNI/NIE + SMS code, like the official app)
    Activate {
        /// Your NIF (DNI/NIE number)
        #[arg(short, long)]
        nif: String,

        /// DNI: card validity date. DNI K/L or NIE M: date of birth. Format DD-MM-YYYY
        #[arg(short, long, default_value = "")]
        fecha: String,

        /// NIE X/Y/Z: support number of the card
        #[arg(short, long, default_value = "")]
        soporte: String,
    },

    /// Show the pending Llave PIN (start the PIN request on the AEAT website first)
    Pin,

    /// Check if a NIF is registered in Llave
    Check {
        /// NIF to check (uses saved NIF if not provided)
        #[arg(short, long)]
        nif: Option<String>,
    },

    /// Authenticate with DNI/NIE data (weak auth)
    #[command(name = "dni-auth")]
    DniAuth {
        /// NIF (DNI/NIE number)
        #[arg(short, long)]
        nif: String,

        /// DNI: card validity date. DNI K/L or NIE M: date of birth. Format DD-MM-YYYY
        #[arg(short, long, default_value = "")]
        fecha: String,

        /// NIE X/Y/Z: support number of the card
        #[arg(short, long, default_value = "")]
        soporte: String,
    },

    /// Show saved session/account info
    Status,

    /// View your Llave account data
    #[command(name = "my-data")]
    MyData,

    /// View operations history (defaults match the app: last 7 days, confirmed only)
    History {
        /// Number of days to look back
        #[arg(short, long, default_value_t = llave_core::auth::HISTORY_DEFAULT_DAYS)]
        days: i64,

        /// Which requests to list
        #[arg(short, long, value_enum, default_value_t = HistoryFilter::Confirmed)]
        result: HistoryFilter,
    },

    /// Deactivate this device
    Deactivate,

    /// Log out and clear saved credentials
    Logout,

    /// Confirm a login shown as a QR code on the AEAT website
    Qr {
        /// QR code text, or the https://sede.agenciatributaria.gob.es/appclave/qrscan?qrcode=… link
        // Long form only: `-v` is the global `--verbose`.
        #[arg(long)]
        value: String,

        /// Skip the security check prompt (answering "no" there rejects the request)
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Listen for pending Llave Móvil authentication requests (polls server)
    Listen {
        /// Polling interval in seconds
        #[arg(short, long, default_value = "5")]
        interval: u64,

        /// Maximum number of polling attempts (0 = unlimited)
        #[arg(short, long, default_value = "60")]
        max_attempts: u32,
    },

    /// Confirm a pending Llave Móvil authentication request
    Confirm {
        /// Llave Móvil token from the pending request
        #[arg(short, long)]
        token: String,

        /// Identity Provider code from the pending request
        #[arg(short = 'i', long)]
        idp_code: String,

        /// Skip the security check prompt (answering "no" there rejects the request)
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Reject a pending Llave Móvil authentication request
    Reject {
        /// Llave Móvil token from the pending request
        #[arg(short, long)]
        token: String,

        /// Identity Provider code from the pending request
        #[arg(short = 'i', long)]
        idp_code: String,
    },

    /// Check for pending authentication requests (single poll)
    Pending,
}

/// `--result` values for `history`.
#[derive(Clone, Copy, clap::ValueEnum)]
enum HistoryFilter {
    All,
    Confirmed,
    Rejected,
}

impl From<HistoryFilter> for llave_core::auth::HistoryResult {
    fn from(f: HistoryFilter) -> Self {
        match f {
            HistoryFilter::All => Self::All,
            HistoryFilter::Confirmed => Self::Confirmed,
            HistoryFilter::Rejected => Self::Rejected,
        }
    }
}

fn output(cli: &Cli, json_value: serde_json::Value) {
    if cli.plain {
        match &json_value {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    match v {
                        serde_json::Value::String(s) => println!("{k}: {s}"),
                        serde_json::Value::Null => println!("{k}: (none)"),
                        other => println!("{k}: {other}"),
                    }
                }
            }
            serde_json::Value::String(s) => println!("{s}"),
            other => println!("{other}"),
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&json_value).unwrap());
    }
}

/// Read one trimmed line from stdin, printing `message` on stderr first.
fn prompt(message: &str) -> Result<String> {
    use std::io::Write;
    eprint!("{message}");
    std::io::stderr().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

/// Pick the `(FECHA, SOPORTE)` values the document type needs, like the app:
/// only the relevant field is sent, the other one goes empty.
fn dni_fields(nif: &str, fecha: &str, soporte: &str) -> Result<(String, String)> {
    use llave_core::config::{required_dni_input, DniInput};
    let missing = |what: &str| LlaveError::Config(format!("{nif} requires {what}"));
    match required_dni_input(nif) {
        DniInput::ValidityDate if fecha.is_empty() => {
            Err(missing("--fecha with the ID card validity date (DD-MM-YYYY)"))
        }
        DniInput::BirthDate if fecha.is_empty() => {
            Err(missing("--fecha with your date of birth (DD-MM-YYYY)"))
        }
        DniInput::SupportNumber if soporte.is_empty() => {
            Err(missing("--soporte with the support number of your NIE card"))
        }
        DniInput::ValidityDate | DniInput::BirthDate => Ok((fecha.to_string(), String::new())),
        DniInput::SupportNumber => Ok((String::new(), soporte.to_string())),
    }
}

/// Location of the PIN-sealed session file.
fn sealed_session_path() -> Result<std::path::PathBuf> {
    Ok(llave_core::config::data_dir()?.join("session.sealed"))
}

/// Ask for the PIN on the terminal without echo. A new PIN is entered twice.
fn prompt_pin(request: PinRequest) -> Result<zeroize::Zeroizing<String>> {
    let read = |message: &str| -> Result<zeroize::Zeroizing<String>> {
        Ok(zeroize::Zeroizing::new(rpassword::prompt_password(message)?))
    };
    match request {
        PinRequest::Create => {
            let min = llave_core::storage::MIN_PIN_LEN;
            for _ in 0..llave_core::storage::UNLOCK_ATTEMPTS {
                let pin = read(&format!(
                    "Choose a PIN to protect your Llave credentials (min {min} characters): "
                ))?;
                if pin.chars().count() < min {
                    eprintln!("The PIN must have at least {min} characters.");
                } else if *read("Repeat the PIN: ")? != *pin {
                    eprintln!("The PINs do not match.");
                } else {
                    return Ok(pin);
                }
            }
            Err(LlaveError::Config("No PIN chosen".into()))
        }
        PinRequest::Unlock { attempt } => read(&format!(
            "Llave PIN ({attempt}/{}): ",
            llave_core::storage::UNLOCK_ATTEMPTS
        )),
    }
}

/// Whether a prompt answer means "yes" (`y`/`yes`/`s`/`si`/`sí`, any case).
fn is_yes(answer: &str) -> bool {
    matches!(
        answer.trim().to_lowercase().as_str(),
        "y" | "yes" | "s" | "si" | "sí"
    )
}

/// The app's pre-confirmation check: show the verification code and ask
/// whether the user started the login on the official AEAT website.
fn confirm_security_check(codigo_idp: &str) -> Result<bool> {
    eprintln!("Verification code: {codigo_idp}");
    eprintln!("It must match the code shown on the website where you are logging in.");
    eprintln!("Only continue if you started this login yourself, not from a link in an email or SMS,");
    eprintln!("and the website address starts with https://www2. or https://www12.agenciatributaria.gob.es");
    let answer = prompt("Confirm this request? Answering no rejects it [y/N]: ")?;
    Ok(is_yes(&answer))
}

/// Show the server's request and ask for the date of birth (DD-MM-YYYY).
fn prompt_birth_date(server_message: &str) -> Result<String> {
    if !server_message.is_empty() {
        eprintln!("{server_message}");
    }
    let birth_date = prompt("Date of birth (DD-MM-YYYY): ")?;
    if birth_date.is_empty() {
        return Err(LlaveError::Config("Date of birth is required".into()));
    }
    Ok(birth_date)
}

fn output_error(cli: &Cli, err: &LlaveError) {
    if let LlaveError::IpRateLimited { message } = err {
        eprintln!();
        eprintln!("  ╔══════════════════════════════════════════════════════════════╗");
        eprintln!("  ║  IP TEMPORARILY BLOCKED BY AEAT                              ║");
        eprintln!("  ╠══════════════════════════════════════════════════════════════╣");
        eprintln!("  ║                                                              ║");
        eprintln!("  ║  Your IP address has exceeded the maximum number of failed   ║");
        eprintln!("  ║  attempts allowed per day. You can try again tomorrow.       ║");
        eprintln!("  ║                                                              ║");
        eprintln!("  ║  Tip: Use a VPN or proxy to switch IP addresses:             ║");
        eprintln!("  ║    llave --proxy socks5://127.0.0.1:1080 <command>           ║");
        eprintln!("  ║                                                              ║");
        eprintln!("  ╚══════════════════════════════════════════════════════════════╝");
        eprintln!();
        if cli.verbose {
            eprintln!("  Server message: {message}");
            eprintln!();
        }
        return;
    }

    if cli.plain {
        eprintln!("Error: {err}");
    } else {
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "error": err.to_string()
            }))
            .unwrap()
        );
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let filter = if cli.verbose {
        "llave=debug,llave_core=debug,info"
    } else {
        "llave=info,llave_core=info,warn"
    };
    tracing_subscriber::Registry::default()
        .with(tracing_logfmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| filter.parse().unwrap()),
        )
        .init();

    // Durable PIN-sealed file; the kernel keyring only caches the unlocked
    // session for this login (it does not survive a reboot).
    match sealed_session_path() {
        Ok(path) => init_storage(Box::new(SealedFileStorage::new(
            path,
            Some(Box::new(KeyringStorage::new())),
            Box::new(prompt_pin),
        ))),
        Err(e) => {
            output_error(&cli, &e);
            std::process::exit(1);
        }
    }

    // Set proxy: CLI flag > config file > env var (LLAVE_PROXY handled by clap)
    let proxy = cli
        .proxy
        .clone()
        .or_else(|| Config::load().ok().and_then(|c| c.proxy));
    if proxy.is_some() {
        llave_core::set_proxy(proxy);
    }

    if let Err(e) = run(&cli).await {
        output_error(&cli, &e);
        std::process::exit(1);
    }
}

async fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Commands::Activate {
            nif,
            fecha,
            soporte,
        } => {
            let nif = validate_nif(nif)?;
            let (fecha, soporte) = dni_fields(&nif, fecha, soporte)?;
            // Activation always starts from no session, so the new
            // credentials are sealed with a freshly chosen PIN.
            if sealed_session_path()?.exists() {
                return Err(LlaveError::Config(
                    "This device is already activated. Run `llave deactivate` (or `llave logout`) first.".into(),
                ));
            }
            // Choose the local PIN first: once the server activates the device,
            // a PIN typo must not be able to lose the new credentials.
            llave_core::storage::prepare_new_session()?;
            let client = LlaveClient::new()?;

            // Phase 1: DNI/NIE auth → registration check → SMS sent.
            // Like the app, retry once with the date of birth if asked.
            let phase1 =
                match llave_core::auth::dni_request_sms(&client, &nif, &fecha, &soporte, None).await {
                    Err(LlaveError::DniBirthDateRequired { challenge, message }) => {
                        let birth_date = prompt_birth_date(&message)?;
                        llave_core::auth::dni_request_sms(
                            &client,
                            &nif,
                            &birth_date,
                            "",
                            Some(challenge),
                        )
                        .await?
                    }
                    other => other?,
                };

            // Prompt on stderr so stdout stays machine-readable.
            let sms_pin = prompt(&format!(
                "SMS code sent to {} — enter code: ",
                phase1.sms.movil
            ))?;
            if sms_pin.is_empty() {
                return Err(LlaveError::Config("SMS code is required".into()));
            }

            // Phase 2: validate SMS code → activate device (www6).
            let device_password = llave_core::crypto::generate_device_password();
            let session = llave_core::auth::dni_validate_and_activate(
                &client,
                &phase1.cookies_json,
                &nif,
                &phase1.sms.timestamp_alta_sms,
                &phase1.sms.token_clave_movil_sms,
                &sms_pin,
                &device_password,
            )
            .await?;

            let mut cfg = Config::load()?;
            cfg.nif = Some(nif.clone());
            cfg.device_id = Some(session.device_id.clone());
            cfg.save()?;

            output(
                cli,
                json!({
                    "status": "activated",
                    "nif": session.nif,
                    "device_id": session.device_id,
                    "created_at": session.created_at,
                }),
            );
        }

        Commands::Pin => {
            let session = Session::load()?;
            let client = LlaveClient::new()?;
            let (pin, ttl) = llave_core::auth::request_pin(&client, &session).await?;

            output(
                cli,
                json!({
                    "pin": pin,
                    "time_to_live_seconds": ttl,
                    "nif": session.nif,
                }),
            );
        }

        Commands::Check { nif } => {
            let nif = if let Some(n) = nif {
                validate_nif(n)?
            } else {
                let cfg = Config::load()?;
                cfg.nif.ok_or(LlaveError::Config(
                    "No NIF configured. Use --nif or run `llave activate` first.".into(),
                ))?
            };

            let client = LlaveClient::new()?;
            // Like the app: the stored device id (empty if not activated).
            let device_id = Session::load().map(|s| s.device_id).unwrap_or_default();

            let resp = client.clave_is_nif_activated(&device_id, &nif).await?;

            output(
                cli,
                json!({
                    "nif": nif,
                    "status": resp.status,
                    "response": resp.respuesta,
                }),
            );
        }

        Commands::DniAuth {
            nif,
            fecha,
            soporte,
        } => {
            let nif = validate_nif(nif)?;
            let (fecha, soporte) = dni_fields(&nif, fecha, soporte)?;
            let client = LlaveClient::new()?;
            match llave_core::auth::authenticate_dni(&client, &nif, &fecha, &soporte, None).await {
                Err(LlaveError::DniBirthDateRequired { challenge, message }) => {
                    let birth_date = prompt_birth_date(&message)?;
                    llave_core::auth::authenticate_dni(&client, &nif, &birth_date, "", Some(challenge))
                        .await?;
                }
                other => other?,
            }

            output(
                cli,
                json!({
                    "nif": nif,
                    "auth_type": "dni_nie_weak",
                    "status": "OK",
                    "message": "Session cookies established",
                }),
            );
        }

        Commands::Status => {
            match Session::load() {
                Ok(session) => {
                    output(
                        cli,
                        json!({
                            "active": true,
                            "nif": session.nif,
                            "device_id": session.device_id,
                            "created_at": session.created_at,
                            "has_firebase_token": session.firebase_token.is_some(),
                        }),
                    );
                }
                Err(_) => {
                    output(
                        cli,
                        json!({
                            "active": false,
                            "message": "No active session. Run `llave activate` to set up."
                        }),
                    );
                }
            };
        }

        Commands::MyData => {
            let session = Session::load()?;
            let client = LlaveClient::new()?;

            let data = llave_core::auth::my_data(&client, &session).await?;

            output(
                cli,
                json!({
                    "status": "OK",
                    "nif": session.nif,
                    "data": data,
                }),
            );
        }

        Commands::History { days, result } => {
            let session = Session::load()?;
            let client = LlaveClient::new()?;

            let operations =
                llave_core::auth::history(&client, &session, *days, (*result).into()).await?;

            output(
                cli,
                json!({
                    "status": "OK",
                    "operations": operations,
                }),
            );
        }

        Commands::Deactivate => {
            let session = Session::load()?;
            let client = LlaveClient::new()?;

            llave_core::auth::deactivate(&client, &session).await?;

            output(
                cli,
                json!({
                    "status": "OK",
                    "message": "Device deactivated",
                }),
            );
        }

        Commands::Logout => {
            Session::delete()?;
            output(
                cli,
                json!({
                    "status": "logged_out",
                    "message": "Session cleared"
                }),
            );
        }

        Commands::Qr { value, yes } => {
            let session = Session::load()?;
            let client = LlaveClient::new()?;

            // Like the app: the QR yields a pending request, which the user
            // then confirms (ClaveAuthenticateSv) or rejects.
            let respuesta = llave_core::auth::qr_authenticate(&client, &session, value).await?;
            let peticion = respuesta.get("peticion");
            let field = |name: &str| {
                peticion
                    .and_then(|p| p.get(name))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string()
            };
            let (token, idp_code) = (field("tokenClaveMovil"), field("codigoIdP"));
            if token.is_empty() {
                return Err(LlaveError::Api {
                    status: "OK".into(),
                    code: String::new(),
                    message: "QR response has no request to confirm".into(),
                });
            }
            eprintln!("Request from: {}", field("organismo"));

            let result = if *yes || confirm_security_check(&idp_code)? {
                llave_core::auth::confirm_authentication(&client, &session, &token, &idp_code)
                    .await?
            } else {
                let rejected =
                    llave_core::auth::reject_authentication(&client, &session, &token, &idp_code)
                        .await?;
                json!({ "rejected": true, "response": rejected })
            };
            output(cli, result);
        }

        Commands::Listen {
            interval,
            max_attempts,
        } => {
            let session = Session::load()?;
            let client = LlaveClient::new()?;

            if !cli.plain {
                eprintln!("Listening for pending Llave Móvil requests (polling every {interval}s, max {max_attempts} attempts)...");
                eprintln!("Press Ctrl+C to stop.");
            }

            let result =
                llave_core::auth::listen_for_requests(&client, &session, *interval, *max_attempts)
                    .await?;

            output(cli, result);
        }

        Commands::Confirm {
            token,
            idp_code,
            yes,
        } => {
            let session = Session::load()?;
            let client = LlaveClient::new()?;

            // Same security check as the app: "No" rejects the request.
            if !yes && !confirm_security_check(idp_code)? {
                let result =
                    llave_core::auth::reject_authentication(&client, &session, token, idp_code)
                        .await?;
                output(cli, json!({ "rejected": true, "response": result }));
                return Ok(());
            }

            let result =
                llave_core::auth::confirm_authentication(&client, &session, token, idp_code)
                    .await?;

            output(cli, result);
        }

        Commands::Reject { token, idp_code } => {
            let session = Session::load()?;
            let client = LlaveClient::new()?;

            let result =
                llave_core::auth::reject_authentication(&client, &session, token, idp_code).await?;

            output(cli, result);
        }

        Commands::Pending => {
            let session = Session::load()?;
            let client = LlaveClient::new()?;

            let result = llave_core::auth::poll_pending_requests(&client, &session).await?;

            output(cli, result);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_definition_is_valid() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    #[test]
    fn yes_accepts_spanish_and_english() {
        assert!(["y", "YES", " sí ", "si", "S"].iter().all(|a| is_yes(a)));
    }

    #[test]
    fn yes_rejects_empty_answer() {
        assert!(!is_yes(""));
    }

    #[test]
    fn yes_rejects_no() {
        assert!(!is_yes("n"));
    }

    #[test]
    fn dni_fields_dni_sends_only_fecha() {
        let fields = dni_fields("12345678Z", "01-01-2030", "ABC").unwrap();
        assert_eq!(fields, ("01-01-2030".to_string(), String::new()));
    }

    #[test]
    fn dni_fields_nie_sends_only_soporte() {
        let fields = dni_fields("X1234567L", "01-01-2030", "E12345678").unwrap();
        assert_eq!(fields, (String::new(), "E12345678".to_string()));
    }

    #[test]
    fn dni_fields_nie_without_soporte_fails() {
        assert!(dni_fields("X1234567L", "01-01-2030", "").is_err());
    }

    #[test]
    fn dni_fields_dni_without_fecha_fails() {
        assert!(dni_fields("12345678Z", "", "ABC").is_err());
    }
}
