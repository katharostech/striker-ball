use crate::*;
use bones::*;
use bones_framework::networking::*;

impl SessionPlugin for Matchmaker {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
        session.add_system_to_stage(
            First,
            |time: Res<Time>, mut matchmaker: ResMut<Matchmaker>| {
                matchmaker.update(time.delta());
            },
        );
    }
}

#[derive(HasSchema, Clone)]
#[schema(no_default)]
pub struct Matchmaker {
    service_name: String,

    // Host
    pub host_name: String,
    player_count: u32,
    server: Option<lan::ServerInfo>,
    joined_players: usize,

    // Search
    pub search_enabled: bool,
    refresh: Timer,
    lan_servers: Vec<lan::ServerInfo>,
    lan_discovery: Option<lan::ServiceDiscoveryReceiver>,

    // Join
    socket: Option<NetworkMatchSocket>,
}

impl Matchmaker {
    /// Whether or not the matchmaker has created a server for hosting.
    pub fn is_hosting(&self) -> bool {
        self.server.is_some()
    }
    /// This is whether or not the matchmaker has gotten a network socket for a match full of players
    /// either by hosting or by searching then joining.
    pub fn is_joined(&self) -> bool {
        self.socket.is_some()
    }
    pub fn service_name(&self) -> &str {
        &self.service_name
    }
    pub fn service_type(&self) -> String {
        format!("_{}._udp.local.", self.service_name)
    }
    pub fn lan_servers(&self) -> &Vec<lan::ServerInfo> {
        &self.lan_servers
    }
    pub fn joined_players(&self) -> Option<usize> {
        self.is_hosting().then_some(self.joined_players)
    }
    pub fn network_match_socket(&self) -> Option<NetworkMatchSocket> {
        self.socket.clone()
    }
    pub fn disable_search(&mut self) {
        self.search_enabled = false;
    }
    pub fn enable_search(&mut self) {
        self.search_enabled = true;
    }
    pub fn update_service_name(&mut self, name: &str) {
        if self.service_name != name {
            self.service_name = name.to_string();
            self.lan_cancel();
        }
    }
    pub fn lan_host(&mut self) {
        let service_type = self.service_type();
        let (is_recreated, server) = RUNTIME.block_on(async {
            lan::prepare_to_host(&mut self.server, &service_type, &self.host_name).await
        });
        dbg!(is_recreated);

        lan::start_server(server.clone(), self.player_count);

        self.socket = lan::wait_players(&mut self.joined_players, server);
    }
    pub fn lan_join(&mut self, server: &lan::ServerInfo) {
        self.lan_cancel();
        lan::join_server(server).expect("failed to join lan server");
        self.socket = lan::wait_game_start();
    }
    pub fn lan_cancel(&mut self) {
        if let Some(server) = self.server.take() {
            lan::stop_server(&server);
        } else {
            lan::leave_server();
        }
        self.socket = None;
        self.lan_discovery = None;
        self.lan_servers = Vec::new();
    }
    pub fn lan_search(&mut self) {
        let service_type = self.service_type();
        lan::prepare_to_join(
            &service_type,
            &mut self.lan_servers,
            &mut self.lan_discovery,
            &self.refresh,
        );
    }
    pub fn update(&mut self, delta: std::time::Duration) {
        self.refresh.tick(delta);

        if self.search_enabled && !self.is_hosting() && !self.is_joined() && self.refresh.finished()
        {
            tracing::debug!("matchmaker refresh...");
            self.lan_search();
            self.refresh.reset();
        }
        if !self.is_joined() {
            self.socket =
            // is hosting
            if let Some(server) = &self.server {
                lan::wait_players(&mut self.joined_players, server)
                // is joining
            } else {
                lan::wait_game_start()
            };
            // TODO: cancel search if necessary
        }
    }
}

pub struct MatchmakerPlugin {
    pub service_name: String,
    pub host_name: String,
    pub player_count: u32,
    pub refresh: f32,
    pub start_searching: bool,
}
impl MatchmakerPlugin {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            host_name: String::from("default_host"),
            player_count: 2,
            refresh: 2.0,
            start_searching: false,
        }
    }
    pub fn service_name(mut self, service_name: String) -> Self {
        self.service_name = service_name;
        self
    }
    pub fn host_name(mut self, host_name: String) -> Self {
        self.host_name = host_name;
        self
    }
    pub fn player_count(mut self, player_count: u32) -> Self {
        self.player_count = player_count;
        self
    }
    pub fn refresh(mut self, seconds: f32) -> Self {
        self.refresh = seconds;
        self
    }
    pub fn start_searching(mut self) -> Self {
        self.start_searching = true;
        self
    }
}
impl From<MatchmakerPlugin> for Matchmaker {
    fn from(plugin: MatchmakerPlugin) -> Self {
        Matchmaker {
            search_enabled: plugin.start_searching,
            refresh: Timer::from_seconds(plugin.refresh, TimerMode::Once),
            service_name: plugin.service_name,
            host_name: plugin.host_name,
            player_count: plugin.player_count,
            server: None,
            joined_players: 0,
            lan_servers: Vec::new(),
            lan_discovery: None,
            socket: None,
        }
    }
}
impl SessionPlugin for MatchmakerPlugin {
    fn install(self, session: &mut SessionBuilder) {
        session.install_plugin(Matchmaker::from(self));
    }
}
