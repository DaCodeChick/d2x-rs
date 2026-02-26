#ifndef DLE_TYPES_H
#define DLE_TYPES_H

#include <cstdint>
#include <cmath>

namespace dle {

// Fixed-point math (16.16 format, original Descent)
using fix = int32_t;

// Fixed-point conversion functions
inline constexpr double fixToDouble(fix x) {
    return static_cast<double>(x) / 65536.0;
}

inline constexpr fix doubleToFix(double d) {
    return static_cast<fix>(std::round(d * 65536.0));
}

inline constexpr int fixToInt(fix x) {
    return x / 65536;
}

inline constexpr fix intToFix(int i) {
    return static_cast<fix>(i) * 65536;
}

inline constexpr fix fixMul(fix a, fix b) {
    return static_cast<fix>((static_cast<int64_t>(a) * b) / 65536);
}

inline constexpr fix fixDiv(fix a, fix b) {
    return static_cast<fix>((static_cast<int64_t>(a) * 65536) / b);
}

// Basic integer types
using sbyte = int8_t;
using ubyte = uint8_t;
using ushort = uint16_t;
using uint = uint32_t;

// Generic 3D vector template
template<typename T>
struct Vec3 {
    T x, y, z;

    // Constructors
    constexpr Vec3() : x(T(0)), y(T(0)), z(T(0)) {}
    constexpr Vec3(T x_, T y_, T z_) : x(x_), y(y_), z(z_) {}
    
    // Conversion from other Vec3 types
    template<typename U>
    explicit Vec3(const Vec3<U>& other);
    
    // Basic arithmetic operators
    constexpr Vec3 operator+(const Vec3& other) const {
        return Vec3(x + other.x, y + other.y, z + other.z);
    }
    
    constexpr Vec3 operator-(const Vec3& other) const {
        return Vec3(x - other.x, y - other.y, z - other.z);
    }
    
    constexpr Vec3 operator-() const {
        return Vec3(-x, -y, -z);
    }
    
    // Scalar multiplication (generic)
    constexpr Vec3 operator*(T scalar) const {
        return Vec3(x * scalar, y * scalar, z * scalar);
    }
    
    constexpr Vec3 operator/(T scalar) const {
        return Vec3(x / scalar, y / scalar, z / scalar);
    }
    
    // Compound assignment
    constexpr Vec3& operator+=(const Vec3& other) {
        x += other.x;
        y += other.y;
        z += other.z;
        return *this;
    }
    
    constexpr Vec3& operator-=(const Vec3& other) {
        x -= other.x;
        y -= other.y;
        z -= other.z;
        return *this;
    }
    
    constexpr Vec3& operator*=(T scalar) {
        x *= scalar;
        y *= scalar;
        z *= scalar;
        return *this;
    }
    
    // Comparison
    constexpr bool operator==(const Vec3& other) const {
        return x == other.x && y == other.y && z == other.z;
    }
    
    constexpr bool operator!=(const Vec3& other) const {
        return !(*this == other);
    }
    
    // Dot product
    constexpr T dot(const Vec3& other) const {
        return x * other.x + y * other.y + z * other.z;
    }
    
    // Cross product
    constexpr Vec3 cross(const Vec3& other) const {
        return Vec3(
            y * other.z - z * other.y,
            z * other.x - x * other.z,
            x * other.y - y * other.x
        );
    }
    
    // Length operations (only for floating-point types)
    T length() const requires std::is_floating_point_v<T> {
        return std::sqrt(x * x + y * y + z * z);
    }
    
    T lengthSquared() const {
        return x * x + y * y + z * z;
    }
    
    Vec3 normalized() const requires std::is_floating_point_v<T> {
        T len = length();
        return (len > T(0)) ? (*this) / len : *this;
    }
};

// Specialization for fix type (uses fixMul for multiplication)
template<>
inline constexpr Vec3<fix> Vec3<fix>::operator*(fix scalar) const {
    return Vec3<fix>(fixMul(x, scalar), fixMul(y, scalar), fixMul(z, scalar));
}

template<>
inline constexpr Vec3<fix>& Vec3<fix>::operator*=(fix scalar) {
    x = fixMul(x, scalar);
    y = fixMul(y, scalar);
    z = fixMul(z, scalar);
    return *this;
}

// Conversion constructors
template<>
template<>
inline Vec3<double>::Vec3(const Vec3<fix>& other)
    : x(fixToDouble(other.x))
    , y(fixToDouble(other.y))
    , z(fixToDouble(other.z)) {}

template<>
template<>
inline Vec3<fix>::Vec3(const Vec3<double>& other)
    : x(doubleToFix(other.x))
    , y(doubleToFix(other.y))
    , z(doubleToFix(other.z)) {}

// Type aliases for compatibility
using Vector = Vec3<fix>;
using DoubleVector = Vec3<double>;

// Generic 3x3 matrix template (orientation - right, up, forward vectors)
template<typename T>
struct Mat3 {
    Vec3<T> right;
    Vec3<T> up;
    Vec3<T> forward;

    // Default constructor - identity matrix
    constexpr Mat3();
    
    // Construct from three vectors
    constexpr Mat3(const Vec3<T>& r, const Vec3<T>& u, const Vec3<T>& f)
        : right(r), up(u), forward(f) {}
    
    // Conversion from other Mat3 types
    template<typename U>
    explicit Mat3(const Mat3<U>& other)
        : right(Vec3<T>(other.right))
        , up(Vec3<T>(other.up))
        , forward(Vec3<T>(other.forward)) {}
};

// Specialization for fix type - identity uses intToFix
template<>
inline constexpr Mat3<fix>::Mat3()
    : right(intToFix(1), 0, 0)
    , up(0, intToFix(1), 0)
    , forward(0, 0, intToFix(1)) {}

// Specialization for double type - identity uses 1.0
template<>
inline constexpr Mat3<double>::Mat3()
    : right(1.0, 0.0, 0.0)
    , up(0.0, 1.0, 0.0)
    , forward(0.0, 0.0, 1.0) {}

// Type aliases for compatibility
using Matrix = Mat3<fix>;
using DoubleMatrix = Mat3<double>;

// UV Coordinates
struct UVCoord {
    fix u, v;

    constexpr UVCoord() : u(0), v(0) {}
    constexpr UVCoord(fix u_, fix v_) : u(u_), v(v_) {}
};

// UV + Light (per vertex on a side)
struct UVLS {
    fix u, v;
    uint16_t light;

    constexpr UVLS() : u(0), v(0), light(0) {}
    constexpr UVLS(fix u_, fix v_, uint16_t l) : u(u_), v(v_), light(l) {}
};

// File types
enum class FileType {
    RDL,     // Descent 1
    RL2,     // Descent 2
    D2X_XL   // D2X-XL extended
};

// Level versions
constexpr int LEVEL_VERSION_D1 = 1;
constexpr int LEVEL_VERSION_D2 = 8;
constexpr int LEVEL_VERSION_D2X = 9;
constexpr int LEVEL_VERSION_CURRENT = 11;

// Limits
constexpr int MAX_SEGMENTS_D1 = 900;
constexpr int MAX_SEGMENTS_D2 = 900;
constexpr int MAX_SEGMENTS_D2X = 32000;

constexpr int MAX_VERTICES_D1 = 3600;
constexpr int MAX_VERTICES_D2 = 3600;
constexpr int MAX_VERTICES_D2X = 128000;

constexpr int MAX_WALLS_D1 = 175;
constexpr int MAX_WALLS_D2 = 254;
constexpr int MAX_WALLS_D2X = 32000;

constexpr int MAX_OBJECTS = 350;
constexpr int MAX_OBJECTS_D2X = 5000;

constexpr int MAX_TRIGGERS = 100;
constexpr int MAX_TRIGGERS_D2X = 1000;

constexpr int MAX_MATCENS = 20;  // Maximum robot/equipment generators

// Segment constants
constexpr int NUM_SIDES = 6;
constexpr int NUM_VERTICES_PER_SEGMENT = 8;
constexpr int NUM_VERTICES_PER_SIDE = 4;

// Side indices (cube faces)
enum SideIndex : uint8_t {
    SIDE_RIGHT = 0,
    SIDE_TOP = 1,
    SIDE_FRONT = 2,
    SIDE_LEFT = 3,
    SIDE_BOTTOM = 4,
    SIDE_BACK = 5
};

// Segment special types
enum SegmentType : uint8_t {
    SEGMENT_NORMAL = 0,
    SEGMENT_MATCEN = 1,      // Robot generator
    SEGMENT_GOAL_BLUE = 2,   // Blue goal (CTF)
    SEGMENT_GOAL_RED = 3,    // Red goal (CTF)
    SEGMENT_SPEEDBOOST = 7   // Speed boost
};

} // namespace dle

#endif // DLE_TYPES_H
