#ifndef DLE_FILE_WRITER_H
#define DLE_FILE_WRITER_H

#include <QDataStream>
#include <QFile>
#include <QString>
#include <memory>
#include "../types/Types.h"

namespace dle {

/**
 * @brief FileWriter wraps QDataStream for writing Descent level files
 * 
 * Handles binary file writing with proper endianness for:
 * - RDL (Descent 1) - little-endian
 * - RL2 (Descent 2) - little-endian
 * - D2X-XL extensions
 */
class FileWriter {
public:
    FileWriter();
    ~FileWriter();
    
    // Open file for writing
    bool open(const QString& filename);
    void close();
    bool isOpen() const;
    
    // Position
    qint64 pos() const;
    bool seek(qint64 pos);
    
    // Write basic types
    void writeInt8(int8_t value);
    void writeUInt8(uint8_t value);
    void writeInt16(int16_t value);
    void writeUInt16(uint16_t value);
    void writeInt32(int32_t value);
    void writeUInt32(uint32_t value);
    
    // Write fixed-point
    void writeFix(fix value);
    
    // Write arrays
    void writeBytes(const char* data, qint64 len);
    void writeBytes(const QByteArray& data);
    void writeString(const QString& str, qint64 maxLen);
    
    // Write Descent types
    void writeVector(const Vector& vec);
    void writeMatrix(const Matrix& mat);
    void writeUVLS(const UVLS& uvls);
    
    // Padding
    void writePadding(qint64 bytes, uint8_t value = 0);
    
    // Error handling
    bool hasError() const;
    QString errorString() const;
    
    // Flush to disk
    bool flush();

private:
    std::unique_ptr<QFile> m_file;
    std::unique_ptr<QDataStream> m_stream;
};

} // namespace dle

#endif // DLE_FILE_WRITER_H
