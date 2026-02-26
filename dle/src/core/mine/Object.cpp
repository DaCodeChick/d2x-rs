#include "Object.h"
#include <QDataStream>

namespace dle {

namespace {
    // Helper functions for reading/writing Descent types
    Vector readVector(QDataStream& stream) {
        int32_t x, y, z;
        stream >> x >> y >> z;
        return Vector{x, y, z};
    }

    void writeVector(QDataStream& stream, const Vector& v) {
        stream << v.x << v.y << v.z;
    }

    Matrix readMatrix(QDataStream& stream) {
        Matrix m;
        m.right = readVector(stream);
        m.up = readVector(stream);
        m.forward = readVector(stream);
        return m;
    }

    void writeMatrix(QDataStream& stream, const Matrix& m) {
        writeVector(stream, m.right);
        writeVector(stream, m.up);
        writeVector(stream, m.forward);
    }
}

// ===== PhysicsInfo =====

PhysicsInfo::PhysicsInfo()
    : velocity()
    , thrust()
    , mass(intToFix(1))
    , drag(0)
    , brakes(0)
    , rotVelocity()
    , rotThrust()
    , turnRoll(0)
    , flags(0)
{
}

void PhysicsInfo::clear() {
    velocity = Vector();
    thrust = Vector();
    mass = intToFix(1);
    drag = 0;
    brakes = 0;
    rotVelocity = Vector();
    rotThrust = Vector();
    turnRoll = 0;
    flags = 0;
}

void PhysicsInfo::read(QDataStream& stream) {
    velocity = readVector(stream);
    thrust = readVector(stream);
    stream >> mass;
    stream >> drag;
    stream >> brakes;
    rotVelocity = readVector(stream);
    rotThrust = readVector(stream);
    stream >> turnRoll;
    stream >> flags;
}

void PhysicsInfo::write(QDataStream& stream) const {
    writeVector(stream, velocity);
    writeVector(stream, thrust);
    stream << mass;
    stream << drag;
    stream << brakes;
    writeVector(stream, rotVelocity);
    writeVector(stream, rotThrust);
    stream << turnRoll;
    stream << flags;
}

// ===== AIInfo =====

AIInfo::AIInfo()
    : behavior(0)
    , hideSegment(-1)
    , pathLength(0)
{
}

void AIInfo::clear() {
    behavior = 0;
    hideSegment = -1;
    pathLength = 0;
}

void AIInfo::read(QDataStream& stream) {
    stream >> behavior;
    
    // Skip AI flags (11 bytes)
    stream.skipRawData(11);
    
    stream >> hideSegment;
    
    // Skip hide_index (2 bytes)
    stream.skipRawData(2);
    
    stream >> pathLength;
    
    // Skip cur_path_index (2 bytes)
    stream.skipRawData(2);
    
    // Skip follow_path_start_seg, follow_path_end_seg (4 bytes)
    stream.skipRawData(4);
    
    // Skip danger_laser_signature, danger_laser_num (6 bytes)
    stream.skipRawData(6);
}

void AIInfo::write(QDataStream& stream) const {
    stream << behavior;
    
    // Write AI flags (11 bytes of zeros)
    for (int i = 0; i < 11; ++i) {
        stream << static_cast<int8_t>(0);
    }
    
    stream << hideSegment;
    stream << static_cast<int16_t>(0);  // hide_index
    stream << pathLength;
    stream << static_cast<int16_t>(0);  // cur_path_index
    stream << static_cast<int16_t>(0);  // follow_path_start_seg
    stream << static_cast<int16_t>(0);  // follow_path_end_seg
    stream << static_cast<int32_t>(0);  // danger_laser_signature
    stream << static_cast<int16_t>(0);  // danger_laser_num
}

// ===== PowerupInfo =====

PowerupInfo::PowerupInfo()
    : count(0)
{
}

void PowerupInfo::clear() {
    count = 0;
}

void PowerupInfo::read(QDataStream& stream) {
    stream >> count;
}

void PowerupInfo::write(QDataStream& stream) const {
    stream << count;
}

// ===== ObjectContents =====

ObjectContents::ObjectContents()
    : type(-1)
    , id(-1)
    , count(0)
{
}

// ===== Object =====

Object::Object()
    : m_signature(0)
    , m_type(ObjectType::None)
    , m_id(0)
    , m_controlType(ControlType::None)
    , m_movementType(MovementType::None)
    , m_renderType(RenderType::None)
    , m_flags(0)
    , m_multiplayer(0)
    , m_segment(-1)
    , m_position()
    , m_lastPosition()
    , m_orientation()
    , m_size(intToFix(1))
    , m_shields(0)
    , m_contents()
    , m_physicsInfo()
    , m_spinRate()
    , m_aiInfo()
    , m_powerupInfo()
{
}

void Object::clear() {
    m_signature = 0;
    m_type = ObjectType::None;
    m_id = 0;
    m_controlType = ControlType::None;
    m_movementType = MovementType::None;
    m_renderType = RenderType::None;
    m_flags = 0;
    m_multiplayer = 0;
    m_segment = -1;
    m_position = Vector();
    m_lastPosition = Vector();
    m_orientation = Matrix();
    m_size = intToFix(1);
    m_shields = 0;
    m_contents = ObjectContents();
    m_physicsInfo.clear();
    m_spinRate = Vector();
    m_aiInfo.clear();
    m_powerupInfo.clear();
}

void Object::read(QDataStream& stream, int levelVersion) {
    int8_t type_val, id_val, control_val, movement_val, render_val;
    
    // Read basic object info
    stream >> type_val;
    m_type = static_cast<ObjectType>(type_val);
    stream >> id_val;
    m_id = id_val;
    stream >> control_val;
    m_controlType = static_cast<ControlType>(control_val);
    stream >> movement_val;
    m_movementType = static_cast<MovementType>(movement_val);
    stream >> render_val;
    m_renderType = static_cast<RenderType>(render_val);
    stream >> m_flags;
    
    // Multiplayer flag (version > 37 in D2X-XL)
    if (levelVersion > 37) {
        stream >> m_multiplayer;
    } else {
        m_multiplayer = 0;
    }
    
    stream >> m_segment;
    
    // Position and orientation
    m_position = readVector(stream);
    m_orientation = readMatrix(stream);
    
    stream >> m_size;
    stream >> m_shields;
    
    // Last position
    m_lastPosition = readVector(stream);
    
    // Contents (what's inside when destroyed)
    stream >> m_contents.type;
    stream >> m_contents.id;
    stream >> m_contents.count;
    
    // Read movement type specific data
    switch (m_movementType) {
        case MovementType::Physics:
            m_physicsInfo.read(stream);
            break;
            
        case MovementType::Spinning:
            m_spinRate = readVector(stream);
            break;
            
        case MovementType::None:
        default:
            break;
    }
    
    // Read control type specific data
    switch (m_controlType) {
        case ControlType::AI:
            m_aiInfo.read(stream);
            break;
            
        case ControlType::Powerup:
            m_powerupInfo.read(stream);
            break;
            
        case ControlType::Explosion:
            // Skip explosion info (16 bytes)
            stream.skipRawData(16);
            break;
            
        case ControlType::Weapon:
            // Skip weapon/laser info (20 bytes)
            stream.skipRawData(20);
            break;
            
        case ControlType::Light:
            // Skip light info (4 bytes)
            stream.skipRawData(4);
            break;
            
        case ControlType::None:
        case ControlType::Flying:
        case ControlType::Slew:
        case ControlType::Debris:
        case ControlType::ControlCenter:
        default:
            break;
    }
    
    // Read render type specific data
    switch (m_renderType) {
        case RenderType::Polymodel:
            // Skip polymodel info (69 bytes: model=4, angles=60, subobj_flags=4, tmap_override=4, alt_textures=1)
            // But we need to read the model number for m_id
            {
                int32_t modelNum;
                stream >> modelNum;
                // Skip the rest (65 bytes)
                stream.skipRawData(65);
            }
            break;
            
        case RenderType::Fireball:
        case RenderType::VClip:
        case RenderType::WeaponVClip:
            // Skip vclip info (9 bytes)
            stream.skipRawData(9);
            break;
            
        case RenderType::None:
        case RenderType::Laser:
        case RenderType::Hostage:
        case RenderType::Powerup:
        case RenderType::Morphing:
        case RenderType::Debris:
        case RenderType::Smoke:
        default:
            break;
    }
}

void Object::write(QDataStream& stream, int levelVersion) const {
    // Write basic object info
    stream << static_cast<int8_t>(m_type);
    stream << static_cast<int8_t>(m_id);
    stream << static_cast<int8_t>(m_controlType);
    stream << static_cast<int8_t>(m_movementType);
    stream << static_cast<int8_t>(m_renderType);
    stream << m_flags;
    
    // Multiplayer flag (version > 37)
    if (levelVersion > 37) {
        stream << m_multiplayer;
    }
    
    stream << m_segment;
    
    // Position and orientation
    writeVector(stream, m_position);
    writeMatrix(stream, m_orientation);
    
    stream << m_size;
    stream << m_shields;
    
    // Last position
    writeVector(stream, m_lastPosition);
    
    // Contents
    stream << m_contents.type;
    stream << m_contents.id;
    stream << m_contents.count;
    
    // Write movement type specific data
    switch (m_movementType) {
        case MovementType::Physics:
            m_physicsInfo.write(stream);
            break;
            
        case MovementType::Spinning:
            writeVector(stream, m_spinRate);
            break;
            
        case MovementType::None:
        default:
            break;
    }
    
    // Write control type specific data
    switch (m_controlType) {
        case ControlType::AI:
            m_aiInfo.write(stream);
            break;
            
        case ControlType::Powerup:
            m_powerupInfo.write(stream);
            break;
            
        case ControlType::Explosion:
            // Write explosion info (16 bytes of zeros)
            stream << static_cast<int32_t>(0);  // spawn_time
            stream << static_cast<int32_t>(0);  // delete_time
            stream << static_cast<int8_t>(0);   // delete_objnum
            stream << static_cast<int8_t>(0);   // attach_parent
            stream << static_cast<int8_t>(0);   // prev_attach
            stream << static_cast<int8_t>(0);   // next_attach
            break;
            
        case ControlType::Weapon:
            // Write weapon/laser info (20 bytes of zeros)
            stream << static_cast<int16_t>(0);  // parent_type
            stream << static_cast<int16_t>(0);  // parent_num
            stream << static_cast<int32_t>(0);  // parent_signature
            stream << static_cast<int32_t>(0);  // creation_time
            stream << static_cast<int8_t>(0);   // last_hitobj
            stream << static_cast<int8_t>(0);   // track_goal
            stream << static_cast<int16_t>(0);  // padding
            stream << static_cast<int32_t>(0);  // multiplier
            break;
            
        case ControlType::Light:
            // Write light info (4 bytes)
            stream << static_cast<int32_t>(0);  // intensity
            break;
            
        case ControlType::None:
        case ControlType::Flying:
        case ControlType::Slew:
        case ControlType::Debris:
        case ControlType::ControlCenter:
        default:
            break;
    }
    
    // Write render type specific data
    switch (m_renderType) {
        case RenderType::Polymodel:
            // Write polymodel info (69 bytes)
            stream << static_cast<int32_t>(m_id);  // model number
            for (int i = 0; i < 10; ++i) {
                stream << static_cast<int16_t>(0);  // anim_angles
                stream << static_cast<int16_t>(0);
                stream << static_cast<int16_t>(0);
            }
            stream << static_cast<int32_t>(0);    // subobj_flags
            stream << static_cast<int32_t>(-1);   // tmap_override
            stream << static_cast<int8_t>(0);     // alt_textures
            break;
            
        case RenderType::Fireball:
        case RenderType::VClip:
        case RenderType::WeaponVClip:
            // Write vclip info (9 bytes)
            stream << static_cast<int32_t>(m_id);  // vclip_num
            stream << static_cast<int32_t>(0);     // frametime
            stream << static_cast<int8_t>(0);      // framenum
            break;
            
        case RenderType::None:
        case RenderType::Laser:
        case RenderType::Hostage:
        case RenderType::Powerup:
        case RenderType::Morphing:
        case RenderType::Debris:
        case RenderType::Smoke:
        default:
            break;
    }
}

} // namespace dle
