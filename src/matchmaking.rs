use crate::*;
use bones::*;
use bones_framework::networking::*;

impl SessionPlugin for Matchmaker {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
        session.add_system_to_stage(
            First,
            |world: &World, time: Res<Time>, mut matchmaker: ResMut<Matchmaker>| {
                matchmaker.update(time.delta());

                if let Some(socket) = matchmaker.network_match_socket() {
                    world.resources.insert(socket);
                } else {
                    world.resources.remove::<NetworkMatchSocket>();
                }
            },
        );
    }
}

#[derive(HasSchema, Clone)]
#[schema(no_default)]
pub struct Matchmaker {
    service_type: String,

    // Host
    pub host_name: String,
    pub player_count: u32,

    server: Option<lan::ServerInfo>,
    joined_players: usize,

    // Search
    refresh: Timer,
    lan_servers: Vec<lan::ServerInfo>,
    lan_discovery: Option<lan::ServiceDiscoveryReceiver>,

    // Join
    wait: bool,
    socket: Option<NetworkMatchSocket>,
}

// impl builder functions
impl Matchmaker {
    pub fn new(service_type: &str) -> Self {
        Self {
            refresh: Timer::from_seconds(2., TimerMode::Once),
            service_type: format!("_{service_type}._udp.local."),
            host_name: String::from("default_host"),
            player_count: 2,
            server: None,
            joined_players: 0,
            lan_servers: Vec::new(),
            lan_discovery: None,
            wait: false,
            socket: None,
        }
    }
    pub fn refresh(self, seconds: f32) -> Self {
        Self {
            refresh: Timer::from_seconds(seconds, TimerMode::Once),
            ..self
        }
    }
    pub fn host_name(self, name: &str) -> Self {
        Self {
            host_name: name.to_string(),
            ..self
        }
    }
    pub fn player_count(self, player_count: u32) -> Self {
        Self {
            player_count,
            ..self
        }
    }
}

// impl read functions
impl Matchmaker {
    pub fn is_hosting(&self) -> bool {
        self.server.is_some()
    }
    pub fn is_joined(&self) -> bool {
        self.socket.is_some()
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
}

// impl mut functions
impl Matchmaker {
    pub fn lan_host(&mut self) {
        let (is_recreated, server) = RUNTIME.block_on(async {
            lan::prepare_to_host(&mut self.server, &self.service_type, &self.host_name).await
        });

        lan::start_server(server.clone(), self.player_count);

        self.socket = lan::wait_players(&mut self.joined_players, server);
        self.wait = true;
    }
    pub fn lan_host_cancel(&mut self) {
        if let Some(server) = self.server.take() {
            lan::stop_server(&server);
        }
        self.wait = false;
    }
    pub fn lan_search(&mut self) {
        lan::prepare_to_join(
            &self.service_type,
            &mut self.lan_servers,
            &mut self.lan_discovery,
            &self.refresh,
        );
    }
    pub fn lan_join(&mut self, server: &lan::ServerInfo) {
        self.lan_host_cancel();
        lan::join_server(server).expect("failed to join lan server");
        self.socket = lan::wait_game_start();
        self.wait = true;
    }
    pub fn lan_join_cancel(&mut self) {
        if self.is_hosting() {
            self.lan_host_cancel();
        } else {
            lan::leave_server();
        }
        self.socket = None;
        self.wait = false;
    }
    pub fn update(&mut self, delta: std::time::Duration) {
        self.refresh.tick(delta);

        if self.wait {
            self.socket = if let Some(server) = &self.server {
                lan::wait_players(&mut self.joined_players, server)
            } else {
                lan::wait_game_start()
            };
            if self.is_joined() {
                self.wait = false;
            }
        } else if self.refresh.finished() && !self.is_hosting() && !self.is_joined() {
            tracing::debug!("matchmaker refresh...");
            self.lan_search();
            self.refresh.reset();
        }
    }
}
