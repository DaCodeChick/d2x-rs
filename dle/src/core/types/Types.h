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

// Vector types
struct Vector {
    fix x, y, z;

    constexpr Vector() : x(0), y(0), z(0) {}
    constexpr Vector(fix x_, fix y_, fix z_) : x(x_), y(y_), z(z_) {}
    
    constexpr Vector operator+(const Vector& other) const {
        return Vector(x + other.x, y + other.y, z + other.z);
    }
    
    constexpr Vector operator-(const Vector& other) const {
        return Vector(x - other.x, y - other.y, z - other.z);
    }
    
    constexpr Vector operator*(fix scalar) const {
        return Vector(fixMul(x, scalar), fixMul(y, scalar), fixMul(z, scalar));
    }
};

struct DoubleVector {
    double x, y, z;

    constexpr DoubleVector() : x(0.0), y(0.0), z(0.0) {}
    constexpr DoubleVector(double x_, double y_, double z_) : x(x_), y(y_), z(z_) {}
    
    DoubleVector(const Vector& v) 
        : x(fixToDouble(v.x))
        , y(fixToDouble(v.y))
        , z(fixToDouble(v.z)) {}
    
    Vector toVector() const {
        return Vector(doubleToFix(x), doubleToFix(y), doubleToFix(z));
    }
    
    constexpr DoubleVector operator+(const DoubleVector& other) const {
        return DoubleVector(x + other.x, y + other.y, z + other.z);
    }
    
    constexpr DoubleVector operator-(const DoubleVector& other) const {
        return DoubleVector(x - other.x, y - other.y, z - other.z);
    }
    
    constexpr DoubleVector operator*(double scalar) const {
        return DoubleVector(x * scalar, y * scalar, z * scalar);
    }
    
    double length() const {
        return std::sqrt(x * x + y * y + z * z);
    }
    
    DoubleVector normalized() const {
        double len = length();
        return (len > 0.0) ? (*this) * (1.0 / len) : *this;
    }
    
    constexpr double dot(const DoubleVector& other) const {
        return x * other.x + y * other.y + z * other.z;
    }
    
    constexpr DoubleVector cross(const DoubleVector& other) const {
        return DoubleVector(
            y * other.z - z * other.y,
            z * other.x - x * other.z,
            x * other.y - y * other.x
        );
    }
};

// Matrix (orientation - right, up, forward vectors)
struct Matrix {
    Vector right;
    Vector up;
    Vector forward;

    constexpr Matrix() 
        : right(intToFix(1), 0, 0)
        , up(0, intToFix(1), 0)
        , forward(0, 0, intToFix(1)) {}
    
    constexpr Matrix(const Vector& r, const Vector& u, const Vector& f)
        : right(r), up(u), forward(f) {}
};

struct DoubleMatrix {
    DoubleVector right;
    DoubleVector up;
    DoubleVector forward;

    constexpr DoubleMatrix()
        : right(1.0, 0.0, 0.0)
        , up(0.0, 1.0, 0.0)
        , forward(0.0, 0.0, 1.0) {}
    
    constexpr DoubleMatrix(const DoubleVector& r, const DoubleVector& u, const DoubleVector& f)
        : right(r), up(u), forward(f) {}
    
    DoubleMatrix(const Matrix& m)
        : right(m.right)
        , up(m.up)
        , forward(m.forward) {}
    
    Matrix toMatrix() const {
        return Matrix(right.toVector(), up.toVector(), forward.toVector());
    }
};

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
