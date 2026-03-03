#pragma once

#include "../types/Types.h"
#include <cstdint>
#include <string>

class QDataStream;  // Qt forward declaration

namespace dle {

/**
 * @brief Object types
 */
enum class ObjectType : int8_t {
    None = -1,
    Wall = 0,           // Wall (unused in objects array)
    Fireball = 1,       // Fireball/explosion
    Robot = 2,          // Enemy robot
    Hostage = 3,        // Hostage to rescue
    Player = 4,         // Player ship
    Weapon = 5,         // Weapon fire
    Camera = 6,         // Camera
    Powerup = 7,        // Powerup (keys, energy, shields, weapons, etc.)
    Debris = 8,         // Debris
    Reactor = 9,        // Reactor/control center
    Flare = 10,         // Player flare
    Clutter = 11,       // Clutter object
    Ghost = 12,         // Ghost (D2)
    Light = 13,         // Light source (D2)
    Coop = 14,          // Coop player (D2)
    Marker = 15,        // Marker (D2)
    Cambot = 16,        // Camera robot (D2X-XL)
    Monsterball = 17,   // Monsterball (D2X-XL)
    Smoke = 18,         // Smoke effect (D2X-XL)
    Explosion = 19,     // Explosion (D2X-XL)
    Effect = 20,        // Effect (D2X-XL)
};

/**
 * @brief Control types (how object is controlled)
 */
enum class ControlType : uint8_t {
    None = 0,           // No control
    AI = 1,             // AI controlled (robots)
    Explosion = 2,      // Explosion
    Flying = 3,         // Player control (unused in level files)
    Slew = 4,           // Slew control (editor)
    Weapon = 5,         // Weapon (homing, etc.)
    Light = 6,          // Light source
    Debris = 7,         // Debris
    Powerup = 8,        // Powerup
    ControlCenter = 9,  // Control center/reactor
};

/**
 * @brief Movement types
 */
enum class MovementType : uint8_t {
    None = 0,           // No movement
    Physics = 1,        // Physics simulation
    Spinning = 2,       // Spinning in place
};

/**
 * @brief Render types (how object is drawn)
 */
enum class RenderType : uint8_t {
    None = 0,           // Nothing
    Polymodel = 1,      // 3D polygon model
    Fireball = 2,       // Fireball animation
    Laser = 3,          // Laser beam
    Hostage = 4,        // Hostage (special)
    Powerup = 5,        // Powerup (special)
    Morphing = 6,       // Morphing object
    Debris = 7,         // Debris
    Smoke = 8,          // Smoke (D2X-XL)
    VClip = 9,          // Video clip
    WeaponVClip = 10,   // Weapon video clip
};

/**
 * @brief Object flags
 */
enum ObjectFlags : uint8_t {
    ObjectFlagRendered = 0x01,      // Object has been rendered
    ObjectFlagAttached = 0x02,      // Object is attached to wall/segment
};

/**
 * @brief Object physics info (for physics-based movement)
 */
struct PhysicsInfo {
    Vector velocity;        // Velocity vector
    Vector thrust;          // Thrust vector
    fix mass;               // Mass
    fix drag;               // Drag coefficient
    fix brakes;             // Braking force
    Vector rotVelocity;     // Rotational velocity
    Vector rotThrust;       // Rotational thrust
    int16_t turnRoll;       // Turn roll amount
    uint16_t flags;         // Physics flags
    
    PhysicsInfo();
    void clear();
    void read(QDataStream& stream);
    void write(QDataStream& stream) const;
};

/**
 * @brief AI info (for robots)
 */
struct AIInfo {
    uint8_t behavior;       // AI behavior mode
    int16_t hideSegment;    // Segment to hide in
    int16_t pathLength;     // Length of path
    
    AIInfo();
    void clear();
    void read(QDataStream& stream);
    void write(QDataStream& stream) const;
};

/**
 * @brief Powerup info
 */
struct PowerupInfo {
    int32_t count;          // Count (ammo, vulcan cannon, etc.)
    
    PowerupInfo();
    void clear();
    void read(QDataStream& stream);
    void write(QDataStream& stream) const;
};

/**
 * @brief Object contents (what robot contains when destroyed)
 */
struct ObjectContents {
    int8_t type;            // Type of contained object
    int8_t id;              // ID of contained object
    int8_t count;           // How many
    
    ObjectContents();
};

/**
 * @brief Object represents any entity in the level
 * 
 * Objects include:
 * - Robots (enemies)
 * - Powerups (keys, energy, shields, weapons)
 * - Players (start positions)
 * - Hostages
 * - Reactor/control center
 * - Weapons, debris, explosions (runtime only)
 */
class Object {
public:
    Object();
    ~Object() = default;
    
    // Copy/move constructors
    Object(const Object&) = default;
    Object& operator=(const Object&) = default;
    Object(Object&&) = default;
    Object& operator=(Object&&) = default;
    
    void clear();
    
    // Basic properties
    ObjectType getType() const { return m_type; }
    void setType(ObjectType type) { m_type = type; }
    
    int8_t getId() const { return m_id; }
    void setId(int8_t id) { m_id = id; }
    
    ControlType getControlType() const { return m_controlType; }
    void setControlType(ControlType type) { m_controlType = type; }
    
    MovementType getMovementType() const { return m_movementType; }
    void setMovementType(MovementType type) { m_movementType = type; }
    
    RenderType getRenderType() const { return m_renderType; }
    void setRenderType(RenderType type) { m_renderType = type; }
    
    // Location
    int16_t getSegment() const { return m_segment; }
    void setSegment(int16_t segment) { m_segment = segment; }
    
    const Vector& getPosition() const { return m_position; }
    void setPosition(const Vector& pos) { m_position = pos; }
    
    const Matrix& getOrientation() const { return m_orientation; }
    void setOrientation(const Matrix& orient) { m_orientation = orient; }
    
    // Properties
    fix getSize() const { return m_size; }
    void setSize(fix size) { m_size = size; }
    
    fix getShields() const { return m_shields; }
    void setShields(fix shields) { m_shields = shields; }
    
    uint8_t getFlags() const { return m_flags; }
    void setFlags(uint8_t flags) { m_flags = flags; }
    
    uint8_t getMultiplayer() const { return m_multiplayer; }
    void setMultiplayer(uint8_t mp) { m_multiplayer = mp; }
    
    int16_t getSignature() const { return m_signature; }
    void setSignature(int16_t sig) { m_signature = sig; }
    
    // Contents
    const ObjectContents& getContents() const { return m_contents; }
    void setContents(const ObjectContents& contents) { m_contents = contents; }
    
    // Type-specific data
    PhysicsInfo& getPhysicsInfo() { return m_physicsInfo; }
    const PhysicsInfo& getPhysicsInfo() const { return m_physicsInfo; }
    
    AIInfo& getAIInfo() { return m_aiInfo; }
    const AIInfo& getAIInfo() const { return m_aiInfo; }
    
    PowerupInfo& getPowerupInfo() { return m_powerupInfo; }
    const PowerupInfo& getPowerupInfo() const { return m_powerupInfo; }
    
    Vector& getSpinRate() { return m_spinRate; }
    const Vector& getSpinRate() const { return m_spinRate; }
    
    // Type checking helpers
    bool isRobot() const { return m_type == ObjectType::Robot; }
    bool isPowerup() const { return m_type == ObjectType::Powerup; }
    bool isPlayer() const { return m_type == ObjectType::Player; }
    bool isHostage() const { return m_type == ObjectType::Hostage; }
    bool isReactor() const { return m_type == ObjectType::Reactor; }
    bool isWeapon() const { return m_type == ObjectType::Weapon; }
    
    // File I/O
    void read(QDataStream& stream, int levelVersion);
    void write(QDataStream& stream, int levelVersion) const;

private:
    // Basic info
    int16_t m_signature;            // Object signature (unique ID)
    ObjectType m_type;              // Object type
    int8_t m_id;                    // Object ID (which robot, powerup, etc.)
    ControlType m_controlType;      // Control type
    MovementType m_movementType;    // Movement type
    RenderType m_renderType;        // Render type
    uint8_t m_flags;                // Misc flags
    uint8_t m_multiplayer;          // Multiplayer flags
    
    // Location
    int16_t m_segment;              // Segment containing object
    Vector m_position;              // Position in 3D space
    Vector m_lastPosition;          // Last position (for interpolation)
    Matrix m_orientation;           // Orientation matrix
    
    // Properties
    fix m_size;                     // Size (radius)
    fix m_shields;                  // Shields/hit points
    ObjectContents m_contents;      // What's inside (for robots)
    
    // Movement/control data (union in original, separate here for simplicity)
    PhysicsInfo m_physicsInfo;      // Physics movement
    Vector m_spinRate;              // Spinning movement
    AIInfo m_aiInfo;                // AI control
    PowerupInfo m_powerupInfo;      // Powerup control
};

} // namespace dle

