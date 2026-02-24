# Networking Architecture

## Overview

D2X-RS uses a modern client-server networking architecture to replace D2X-XL's outdated peer-to-peer SDL_net implementation. This design supports dedicated servers, prevents cheating, and provides smooth gameplay over modern internet connections.

**Corresponds to**: `network/*.cpp`, `include/network.h`

## Design Goals

1. **Authoritative Server**: Prevent cheating by validating all actions server-side
2. **Low Latency**: Client-side prediction and lag compensation
3. **Scalability**: Support 8-32 players (vs original 8)
4. **Modern Protocols**: TCP/IP and UDP, drop obsolete IPX
5. **NAT Traversal**: Work behind routers without port forwarding
6. **Dedicated Servers**: Headless server binaries
7. **LAN Discovery**: Zero-configuration local multiplayer
8. **Master Server**: Internet game browser

---

## Technology Stack

### Core Networking: bevy_renet

```toml
[dependencies]
bevy_renet = "0.0.14"
renet = "0.0.15"
```

**Why renet**:
- Built for Bevy ECS
- Reliable UDP with automatic retransmission
- Connection management
- Channel-based messaging (reliable, unreliable, ordered)
- Low overhead

### Alternative: QUIC (quinn)

For future consideration:
- Built-in encryption (TLS 1.3)
- Better congestion control
- Multiple streams over single connection

```toml
# Future option
quinn = "0.11"
```

---

## Architecture Overview

```
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│   Client 1  │◄───────►│   Server    │◄───────►│   Client 2  │
│             │         │             │         │             │
│ Prediction  │         │ Authority   │         │ Prediction  │
│ Rendering   │         │ Game Logic  │         │ Rendering   │
└─────────────┘         └─────────────┘         └─────────────┘
       │                       │                       │
       │                       ▼                       │
       │               ┌───────────────┐              │
       └──────────────►│ Master Server │◄─────────────┘
                       │  (Discovery)  │
                       └───────────────┘
```

---

## Server Architecture

### Dedicated Server

```rust
pub struct DedicatedServer {
    server: RenetServer,
    game_state: GameState,
    clients: HashMap<ClientId, ConnectedClient>,
    tick_rate: u32,  // 20-60 Hz
}

pub struct ConnectedClient {
    id: ClientId,
    player_entity: Entity,
    last_input_tick: u32,
    rtt: Duration,  // Round-trip time for lag compensation
}
```

### Server Systems

```rust
fn server_receive_system(
    mut server: ResMut<RenetServer>,
    mut clients: ResMut<ClientRegistry>,
) {
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            let input: PlayerInput = bincode::deserialize(&message).unwrap();
            
            // Store input for processing
            clients.get_mut(&client_id).unwrap()
                .pending_inputs.push(input);
        }
    }
}

fn server_update_system(
    time: Res<Time>,
    mut game_state: ResMut<ServerGameState>,
    clients: Res<ClientRegistry>,
) {
    // Process all client inputs
    for (client_id, client) in clients.iter() {
        for input in &client.pending_inputs {
            apply_player_input(input, client.player_entity, &mut game_state);
        }
        client.pending_inputs.clear();
    }
    
    // Run game simulation
    physics_system(&mut game_state, time.delta());
    ai_system(&mut game_state);
    collision_system(&mut game_state);
    weapon_system(&mut game_state);
}

fn server_send_system(
    mut server: ResMut<RenetServer>,
    game_state: Res<ServerGameState>,
    clients: Res<ClientRegistry>,
) {
    // Create snapshot of game state
    let snapshot = create_snapshot(&game_state);
    let message = bincode::serialize(&snapshot).unwrap();
    
    // Send to all clients
    server.broadcast_message(ServerChannel::StateUpdate, message);
}
```

### Tick Rate

```rust
const SERVER_TICK_RATE: u32 = 20;  // 20 Hz = 50ms per tick
const SERVER_TICK_DURATION: Duration = Duration::from_millis(50);

// In server update loop
fn fixed_update_system(
    mut fixed_time: ResMut<FixedTime>,
) {
    fixed_time.set_period(SERVER_TICK_DURATION);
}
```

---

## Client Architecture

### Client Structure

```rust
pub struct GameClient {
    client: RenetClient,
    predicted_state: ClientGameState,
    last_server_state: ServerSnapshot,
    input_buffer: Vec<PlayerInput>,
    tick: u32,
}
```

### Client-Side Prediction

```rust
fn client_predict_system(
    mut client_state: ResMut<ClientGameState>,
    input: Res<PlayerInput>,
    time: Res<Time>,
) {
    // Apply input immediately for responsiveness
    apply_player_input(&input, &mut client_state);
    
    // Run local physics simulation
    physics_system(&mut client_state, time.delta());
    
    // Store input for server
    client_state.input_buffer.push(input.clone());
}
```

### Server Reconciliation

```rust
fn client_reconcile_system(
    mut client: ResMut<RenetClient>,
    mut client_state: ResMut<ClientGameState>,
) {
    while let Some(message) = client.receive_message(ServerChannel::StateUpdate) {
        let snapshot: ServerSnapshot = bincode::deserialize(&message).unwrap();
        
        // Find our player state in snapshot
        let server_player_state = snapshot.players.get(&client_state.player_id).unwrap();
        
        // Check if our prediction was correct
        let prediction_error = (client_state.player.position - server_player_state.position).length();
        
        if prediction_error > RECONCILIATION_THRESHOLD {
            // Prediction was wrong, correct it
            client_state.player.position = server_player_state.position;
            client_state.player.velocity = server_player_state.velocity;
            
            // Re-apply inputs that haven't been acknowledged yet
            let unconfirmed_inputs = client_state.input_buffer
                .iter()
                .filter(|input| input.tick > snapshot.last_processed_tick);
            
            for input in unconfirmed_inputs {
                apply_player_input(input, &mut client_state);
            }
        }
        
        // Remove acknowledged inputs
        client_state.input_buffer.retain(|input| input.tick > snapshot.last_processed_tick);
        
        // Update other players (interpolate between snapshots)
        update_remote_players(&snapshot, &mut client_state);
    }
}
```

### Entity Interpolation

```rust
fn interpolate_remote_players_system(
    time: Res<Time>,
    mut remote_players: Query<(&RemotePlayer, &mut Transform, &mut Velocity)>,
    client_state: Res<ClientGameState>,
) {
    for (remote, mut transform, mut velocity) in remote_players.iter_mut() {
        // Interpolate between last two snapshots
        let t = client_state.interpolation_time / client_state.snapshot_delta;
        
        transform.translation = remote.previous_state.position.lerp(
            remote.current_state.position,
            t
        );
        
        transform.rotation = remote.previous_state.rotation.slerp(
            remote.current_state.rotation,
            t
        );
    }
}
```

---

## Network Protocol

### Channels

```rust
#[derive(Debug)]
pub enum ClientChannel {
    Input,      // Unreliable, sequenced
    Commands,   // Reliable, ordered
}

#[derive(Debug)]
pub enum ServerChannel {
    StateUpdate,  // Unreliable, sequenced
    Events,       // Reliable, ordered
    Chat,         // Reliable, ordered
}
```

### Message Types

#### Client → Server

```rust
#[derive(Serialize, Deserialize)]
pub struct PlayerInput {
    pub tick: u32,
    pub timestamp: u64,
    pub movement: Vec3,  // Forward, slide, up/down
    pub rotation: Vec3,  // Pitch, heading, bank
    pub fire_primary: bool,
    pub fire_secondary: bool,
    pub fire_flare: bool,
    pub weapon_switch: Option<WeaponType>,
}

#[derive(Serialize, Deserialize)]
pub enum ClientCommand {
    RequestJoin { player_name: String },
    RequestTeam { team: Team },
    ChatMessage { message: String },
    Disconnect,
}
```

#### Server → Client

```rust
#[derive(Serialize, Deserialize)]
pub struct ServerSnapshot {
    pub tick: u32,
    pub last_processed_tick: u32,  // Last client input processed
    pub players: HashMap<PlayerId, PlayerState>,
    pub objects: Vec<ObjectState>,
    pub projectiles: Vec<ProjectileState>,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerState {
    pub player_id: PlayerId,
    pub position: Vec3,
    pub rotation: Quat,
    pub velocity: Vec3,
    pub shields: f32,
    pub energy: f32,
    pub weapon: WeaponType,
}

#[derive(Serialize, Deserialize)]
pub enum ServerEvent {
    PlayerJoined { id: PlayerId, name: String },
    PlayerLeft { id: PlayerId },
    WeaponFired { player: PlayerId, weapon: WeaponType },
    Explosion { position: Vec3, radius: f32 },
    PlayerKilled { victim: PlayerId, killer: PlayerId },
    ObjectDestroyed { object_id: u32 },
}
```

---

## LAN Discovery

### UDP Broadcast

```rust
use std::net::UdpSocket;

const LAN_DISCOVERY_PORT: u16 = 42425;
const BROADCAST_INTERVAL: Duration = Duration::from_secs(1);

pub struct LanDiscovery {
    socket: UdpSocket,
    servers: HashMap<SocketAddr, ServerInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub level: String,
    pub game_mode: GameMode,
    pub players: u8,
    pub max_players: u8,
}

impl LanDiscovery {
    pub fn start_server_broadcast(info: ServerInfo) {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.set_broadcast(true).unwrap();
        
        std::thread::spawn(move || {
            loop {
                let message = bincode::serialize(&ServerAnnouncement {
                    magic: PROTOCOL_MAGIC,
                    version: PROTOCOL_VERSION,
                    info: info.clone(),
                }).unwrap();
                
                socket.send_to(&message, format!("255.255.255.255:{}", LAN_DISCOVERY_PORT)).ok();
                std::thread::sleep(BROADCAST_INTERVAL);
            }
        });
    }
    
    pub fn discover_servers() -> Vec<ServerInfo> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", LAN_DISCOVERY_PORT)).unwrap();
        socket.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        
        let mut servers = HashMap::new();
        let mut buf = [0u8; 1024];
        
        while let Ok((size, addr)) = socket.recv_from(&mut buf) {
            if let Ok(announcement) = bincode::deserialize::<ServerAnnouncement>(&buf[..size]) {
                if announcement.magic == PROTOCOL_MAGIC {
                    servers.insert(addr, announcement.info);
                }
            }
        }
        
        servers.into_values().collect()
    }
}
```

---

## Master Server

### Registration

```rust
pub struct MasterServerClient {
    url: String,
    server_info: ServerInfo,
}

impl MasterServerClient {
    pub async fn register(&self) -> Result<(), NetworkError> {
        let client = reqwest::Client::new();
        
        let response = client
            .post(&format!("{}/register", self.url))
            .json(&self.server_info)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(NetworkError::RegistrationFailed)
        }
    }
    
    pub async fn heartbeat(&self) -> Result<(), NetworkError> {
        let client = reqwest::Client::new();
        
        client
            .post(&format!("{}/heartbeat", self.url))
            .json(&HeartbeatMessage {
                server_id: self.server_info.id,
                players: self.server_info.players,
            })
            .send()
            .await?;
        
        Ok(())
    }
}
```

### Server List

```rust
pub async fn fetch_server_list(master_url: &str) -> Result<Vec<ServerInfo>, NetworkError> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(&format!("{}/servers", master_url))
        .send()
        .await?;
    
    let servers: Vec<ServerInfo> = response.json().await?;
    Ok(servers)
}
```

---

## Lag Compensation

### Server-Side Rewind

```rust
pub struct LagCompensation {
    history: VecDeque<WorldSnapshot>,
    max_history_ms: u64,
}

impl LagCompensation {
    pub fn rewind_to_client_time(&self, client_rtt: Duration) -> WorldSnapshot {
        let target_time = Instant::now() - (client_rtt / 2);
        
        // Find snapshot closest to target time
        self.history.iter()
            .min_by_key(|snapshot| {
                (snapshot.timestamp.duration_since(target_time)).abs()
            })
            .cloned()
            .unwrap_or_else(|| self.history.front().unwrap().clone())
    }
    
    pub fn validate_hit(
        &self,
        shooter_client: &ConnectedClient,
        ray: Ray,
        target: Entity,
    ) -> bool {
        // Rewind world to when shooter fired
        let historical_world = self.rewind_to_client_time(shooter_client.rtt);
        
        // Check if ray hit target at that time
        historical_world.ray_intersect(ray, target)
    }
}
```

---

## Game Modes

### Network Game Mode Support

```rust
pub trait NetworkGameMode {
    fn on_player_join(&mut self, player: Entity);
    fn on_player_leave(&mut self, player: Entity);
    fn on_player_killed(&mut self, victim: Entity, killer: Entity);
    fn on_tick(&mut self, world: &mut World);
    fn get_scores(&self) -> Vec<(PlayerId, i32)>;
}

// Example: Anarchy (Deathmatch)
pub struct AnarchyMode {
    scores: HashMap<PlayerId, i32>,
    kill_limit: Option<i32>,
    time_limit: Option<Duration>,
}

impl NetworkGameMode for AnarchyMode {
    fn on_player_killed(&mut self, victim: Entity, killer: Entity) {
        *self.scores.entry(killer.player_id).or_insert(0) += 1;
        
        // Check win condition
        if let Some(limit) = self.kill_limit {
            if *self.scores.get(&killer.player_id).unwrap() >= limit {
                // End game
            }
        }
    }
}

// Example: Capture the Flag
pub struct CtfMode {
    red_flag: Entity,
    blue_flag: Entity,
    red_score: u8,
    blue_score: u8,
}

// Example: Monsterball (D2X-XL)
pub struct MonsterballMode {
    ball: Entity,
    ball_carrier: Option<Entity>,
    goals: Vec<Entity>,
}
```

---

## Security

### Cheat Prevention

1. **Input Validation**
```rust
fn validate_player_input(input: &PlayerInput) -> Result<(), ValidationError> {
    // Check input values are within reasonable bounds
    if input.movement.length() > MAX_MOVEMENT_SPEED {
        return Err(ValidationError::InvalidMovement);
    }
    
    if input.rotation.length() > MAX_ROTATION_SPEED {
        return Err(ValidationError::InvalidRotation);
    }
    
    // Check input timing
    if input.tick < last_input_tick {
        return Err(ValidationError::OutOfOrderInput);
    }
    
    Ok(())
}
```

2. **Server Authority**
- Server validates all actions (weapon fire, item pickup)
- Client requests, server approves
- No trust in client position (only use as hint)

3. **Anti-Speedhack**
```rust
fn check_movement_validity(
    old_pos: Vec3,
    new_pos: Vec3,
    dt: f32,
    max_speed: f32,
) -> bool {
    let distance = (new_pos - old_pos).length();
    let max_distance = max_speed * dt * 1.1;  // 10% tolerance
    distance <= max_distance
}
```

---

## Bandwidth Optimization

### Delta Compression

```rust
#[derive(Serialize, Deserialize)]
pub struct DeltaSnapshot {
    pub base_tick: u32,
    pub changed_players: Vec<(PlayerId, PlayerStateDelta)>,
    pub new_objects: Vec<ObjectState>,
    pub removed_objects: Vec<ObjectId>,
}

pub struct PlayerStateDelta {
    pub position: Option<Vec3>,
    pub rotation: Option<Quat>,
    pub velocity: Option<Vec3>,
    pub shields: Option<f32>,
    pub energy: Option<f32>,
}
```

### Interest Management

Only send entities relevant to each client:

```rust
fn calculate_relevant_entities(
    player_position: Vec3,
    player_segment: Entity,
    world: &World,
) -> Vec<Entity> {
    let mut relevant = Vec::new();
    
    // Always include own segment and connected segments
    relevant.extend(get_nearby_segments(player_segment, 2));
    
    // Include visible objects
    for entity in world.entities() {
        if is_visible_from(player_position, entity.position) {
            relevant.push(entity);
        }
    }
    
    relevant
}
```

---

## Configuration

```toml
[network]
server_tick_rate = 20
max_players = 16
timeout_seconds = 30

[network.bandwidth]
max_snapshot_size = 1400  # MTU - headers
send_rate = 20  # Hz

[network.lan]
discovery_port = 42425
broadcast_interval = 1  # seconds

[network.master]
url = "https://d2x-rs.example.com/master"
heartbeat_interval = 60  # seconds
```

---

## Testing

### Network Simulation

```rust
pub struct NetworkSimulator {
    latency_ms: u32,
    jitter_ms: u32,
    packet_loss: f32,
}

impl NetworkSimulator {
    pub fn simulate_send(&mut self, packet: Packet) {
        // Random packet loss
        if rand::random::<f32>() < self.packet_loss {
            return;  // Drop packet
        }
        
        // Add latency + jitter
        let delay = self.latency_ms + rand::random::<u32>() % self.jitter_ms;
        
        tokio::time::sleep(Duration::from_millis(delay as u64)).await;
        actually_send(packet);
    }
}
```

---

## Migration from D2X-XL

### Removed Features
- IPX protocol (obsolete)
- Serial/modem connections (obsolete)
- Peer-to-peer mode (replaced with client-server)

### Added Features
- Dedicated server support
- Modern NAT traversal
- Master server for game discovery
- Client-side prediction
- Lag compensation
- Higher player counts (16-32 vs 8)

---

## References

- Valve: Source Multiplayer Networking
- Gaffer on Games: Networked Physics
- bevy_renet documentation
- D2X-XL: `network/*.cpp`

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-23
