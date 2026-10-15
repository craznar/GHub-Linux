pub const BUS_NAME: &str = "au.craznar.LinuxLogitechControl1";
pub const OBJECT_PATH: &str = "/au/craznar/LinuxLogitechControl1";
pub const INTERFACE: &str = BUS_NAME;

use llc_core::Config;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use zbus::{message::Header, zvariant::OwnedObjectPath};

#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "au.craznar.LinuxLogitechControl1.Error")]
pub enum ServiceError {
    InvalidInput(String),
    AuthorizationDenied(String),
    DeviceUnavailable(String),
    Conflict(String),
    #[zbus(error)]
    ZBus(zbus::Error),
}
type Result<T> = std::result::Result<T, ServiceError>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionFacts {
    pub uid: u32,
    pub active: bool,
    pub remote: bool,
    pub locked: bool,
    pub class: String,
    pub kind: String,
    pub seat: String,
}
impl SessionFacts {
    pub fn permitted(&self) -> bool {
        self.active
            && !self.remote
            && !self.locked
            && self.class == "user"
            && matches!(self.kind.as_str(), "wayland" | "x11")
            && self.seat == "seat0"
    }
}
#[derive(Debug, Clone)]
struct Owner {
    uid: u32,
    session: OwnedObjectPath,
}
#[derive(Default)]
pub struct State {
    config: Option<Config>,
    owner: Option<Owner>,
    bank: u8,
    bypass: bool,
}
pub struct Backend {
    pub state: Arc<Mutex<State>>,
}
impl Default for Backend {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(State::default())),
        }
    }
}

async fn authorize(connection: &zbus::Connection, header: &Header<'_>) -> Result<Owner> {
    let sender = header
        .sender()
        .ok_or_else(|| ServiceError::AuthorizationDenied("missing bus sender".into()))?;
    let dbus = zbus::fdo::DBusProxy::new(connection).await?;
    let uid = dbus
        .get_connection_unix_user(sender.clone().into())
        .await
        .map_err(|_| ServiceError::AuthorizationDenied("sender credentials unavailable".into()))?;
    let pid = dbus
        .get_connection_unix_process_id(sender.clone().into())
        .await
        .map_err(|_| ServiceError::AuthorizationDenied("sender PID unavailable".into()))?;
    let manager = zbus::Proxy::new(
        connection,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .await?;
    let session: OwnedObjectPath =
        manager
            .call("GetSessionByPID", &(pid,))
            .await
            .map_err(|_| {
                ServiceError::AuthorizationDenied(
                    "caller has no local graphical logind session".into(),
                )
            })?;
    let proxy = zbus::Proxy::new(
        connection,
        "org.freedesktop.login1",
        session.as_str(),
        "org.freedesktop.login1.Session",
    )
    .await?;
    let (session_uid, _): (u32, OwnedObjectPath) = proxy.get_property("User").await?;
    let (seat, _): (String, OwnedObjectPath) = proxy.get_property("Seat").await?;
    let facts = SessionFacts {
        uid,
        active: proxy.get_property("Active").await?,
        remote: proxy.get_property("Remote").await?,
        locked: proxy.get_property("LockedHint").await?,
        class: proxy.get_property("Class").await?,
        kind: proxy.get_property("Type").await?,
        seat,
    };
    if session_uid != uid || !facts.permitted() {
        return Err(ServiceError::AuthorizationDenied(
            "only the active, unlocked, local graphical user on seat0 may control this service"
                .into(),
        ));
    }
    Ok(Owner { uid, session })
}
fn claim(state: &mut State, owner: Owner) -> Result<()> {
    if state
        .owner
        .as_ref()
        .is_some_and(|current| current.uid != owner.uid || current.session != owner.session)
    {
        return Err(ServiceError::Conflict(
            "another session owns the runtime".into(),
        ));
    }
    state.owner = Some(owner);
    Ok(())
}
fn lock(state: &Mutex<State>) -> Result<std::sync::MutexGuard<'_, State>> {
    state
        .lock()
        .map_err(|_| ServiceError::Conflict("backend state unavailable; restart service".into()))
}

#[zbus::interface(name = "au.craznar.LinuxLogitechControl1")]
impl Backend {
    fn interface_version(&self) -> u32 {
        1
    }
    fn status(&self) -> Result<String> {
        let state = lock(&self.state)?;
        Ok(serde_json::json!({"schema_version":1,"interface_version":1,"state":if state.bypass {"bypass"} else {"ready-pass-through"},"management_enabled":false,"hardware_gate":"G0-unverified","active_revision":null,"staged_revision":state.config.as_ref().map(|c| c.revision),"profile":state.config.as_ref().map(|c| c.selected_profile),"bank":if state.bank == 0 {1} else {state.bank},"recording":false,"reason":"Hardware routes have not been qualified; input management is unavailable in this engineering build."}).to_string())
    }
    fn devices(&self) -> Result<String> {
        let report =
            llc_protocol::probe().map_err(|e| ServiceError::DeviceUnavailable(e.to_string()))?;
        serde_json::to_string(&report).map_err(|e| ServiceError::Conflict(e.to_string()))
    }
    fn validate_configuration(&self, json: &str) -> Result<String> {
        let config = Config::parse(json.as_bytes())
            .map_err(|e| ServiceError::InvalidInput(e.to_string()))?;
        Ok(serde_json::json!({"schema_version":1,"valid":true,"revision":config.revision,"profiles":config.profiles.len(),"macros":config.macros.len()}).to_string())
    }
    async fn stage_configuration(
        &self,
        json: &str,
        #[zbus(connection)] connection: &zbus::Connection,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<()> {
        let owner = authorize(connection, &header).await?;
        let config = Config::parse(json.as_bytes())
            .map_err(|e| ServiceError::InvalidInput(e.to_string()))?;
        let mut state = lock(&self.state)?;
        claim(&mut state, owner)?;
        if state
            .config
            .as_ref()
            .is_some_and(|old| old.revision > config.revision)
        {
            return Err(ServiceError::Conflict(
                "stale configuration revision".into(),
            ));
        }
        state.bank = config
            .profiles
            .iter()
            .find(|p| p.id == config.selected_profile)
            .unwrap()
            .default_bank;
        state.config = Some(config);
        Ok(())
    }
    async fn select_profile(
        &self,
        id: &str,
        #[zbus(connection)] connection: &zbus::Connection,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<()> {
        let owner = authorize(connection, &header).await?;
        let id = uuid::Uuid::parse_str(id)
            .map_err(|_| ServiceError::InvalidInput("invalid profile UUID".into()))?;
        let mut state = lock(&self.state)?;
        claim(&mut state, owner)?;
        let config = state
            .config
            .as_mut()
            .ok_or_else(|| ServiceError::InvalidInput("no staged configuration".into()))?;
        let bank = config
            .profiles
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| ServiceError::InvalidInput("profile not found".into()))?
            .default_bank;
        config.selected_profile = id;
        state.bank = bank;
        Ok(())
    }
    async fn select_bank(
        &self,
        bank: u8,
        #[zbus(connection)] connection: &zbus::Connection,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<()> {
        let owner = authorize(connection, &header).await?;
        if !(1..=3).contains(&bank) {
            return Err(ServiceError::InvalidInput("bank must be 1–3".into()));
        }
        let mut state = lock(&self.state)?;
        claim(&mut state, owner)?;
        state.bank = bank;
        Ok(())
    }
    async fn set_management(
        &self,
        enabled: bool,
        #[zbus(connection)] connection: &zbus::Connection,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<()> {
        let owner = authorize(connection, &header).await?;
        let mut state = lock(&self.state)?;
        claim(&mut state, owner)?;
        if enabled {
            return Err(ServiceError::DeviceUnavailable(
                "G0 hardware routes unverified; management cannot be enabled in this build".into(),
            ));
        }
        state.bypass = false;
        Ok(())
    }
    async fn set_bypass(
        &self,
        enabled: bool,
        #[zbus(connection)] connection: &zbus::Connection,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<()> {
        let owner = authorize(connection, &header).await?;
        let mut state = lock(&self.state)?;
        claim(&mut state, owner)?;
        state.bypass = enabled;
        Ok(())
    }
    async fn start_recording(
        &self,
        #[zbus(connection)] connection: &zbus::Connection,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<()> {
        authorize(connection, &header).await?;
        Err(ServiceError::DeviceUnavailable(
            "recording requires qualified input routes".into(),
        ))
    }
    async fn stop_recording(
        &self,
        #[zbus(connection)] connection: &zbus::Connection,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<()> {
        authorize(connection, &header).await?;
        Ok(())
    }
    fn diagnostics(&self) -> Result<String> {
        Ok(serde_json::json!({"schema_version":1,"status":serde_json::from_str::<serde_json::Value>(&self.status()?).unwrap(),"inventory":serde_json::from_str::<serde_json::Value>(&self.devices()?).unwrap(),"contains":["runtime status","USB topology","report descriptors"],"excludes":["typed keys","macros","serial numbers","user paths"]}).to_string())
    }
}

/// Revokes ownership when the owning session locks, changes, or disappears.
pub fn supervise(connection: &zbus::blocking::Connection, state: &Mutex<State>) {
    let owner = state.lock().ok().and_then(|s| s.owner.clone());
    let Some(owner) = owner else {
        return;
    };
    let valid = (|| -> zbus::Result<bool> {
        let proxy = zbus::blocking::Proxy::new(
            connection,
            "org.freedesktop.login1",
            owner.session.as_str(),
            "org.freedesktop.login1.Session",
        )?;
        let (uid, _): (u32, OwnedObjectPath) = proxy.get_property("User")?;
        let (seat, _): (String, OwnedObjectPath) = proxy.get_property("Seat")?;
        Ok(uid == owner.uid
            && SessionFacts {
                uid,
                active: proxy.get_property("Active")?,
                remote: proxy.get_property("Remote")?,
                locked: proxy.get_property("LockedHint")?,
                class: proxy.get_property("Class")?,
                kind: proxy.get_property("Type")?,
                seat,
            }
            .permitted())
    })()
    .unwrap_or(false);
    if !valid {
        if let Ok(mut state) = state.lock() {
            *state = State::default();
        }
    }
}

pub struct Client {
    connection: zbus::blocking::Connection,
}
impl Client {
    pub fn connect() -> zbus::Result<Self> {
        Ok(Self {
            connection: zbus::blocking::connection::Builder::system()?
                .method_timeout(Duration::from_secs(5))
                .build()?,
        })
    }
    pub fn call<T: serde::Serialize + zbus::zvariant::DynamicType>(
        &self,
        method: &str,
        body: &T,
    ) -> zbus::Result<()> {
        self.connection
            .call_method(Some(BUS_NAME), OBJECT_PATH, Some(INTERFACE), method, body)?;
        Ok(())
    }
    pub fn json(&self, method: &str) -> zbus::Result<String> {
        self.connection
            .call_method(Some(BUS_NAME), OBJECT_PATH, Some(INTERFACE), method, &())?
            .body()
            .deserialize()
    }
    pub fn owner(&self) -> zbus::Result<String> {
        let proxy = zbus::blocking::fdo::DBusProxy::new(&self.connection)?;
        Ok(proxy
            .get_name_owner(zbus::names::BusName::try_from(BUS_NAME)?)
            .map_err(zbus::Error::from)?
            .to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_active_unlocked_local_graphical_user_is_authorized() {
        let good = SessionFacts {
            uid: 1000,
            active: true,
            remote: false,
            locked: false,
            class: "user".into(),
            kind: "wayland".into(),
            seat: "seat0".into(),
        };
        assert!(good.permitted());
        let mut bad = good.clone();
        bad.remote = true;
        assert!(!bad.permitted());
        bad = good.clone();
        bad.active = false;
        assert!(!bad.permitted());
        bad = good.clone();
        bad.locked = true;
        assert!(!bad.permitted());
        bad = good.clone();
        bad.class = "greeter".into();
        assert!(!bad.permitted());
        bad = good.clone();
        bad.kind = "tty".into();
        assert!(!bad.permitted());
        bad = good;
        bad.seat = "seat1".into();
        assert!(!bad.permitted());
    }
}
