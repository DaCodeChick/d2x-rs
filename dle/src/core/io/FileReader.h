#ifndef DLE_FILE_READER_H
#define DLE_FILE_READER_H

#include <QDataStream>
#include <QFile>
#include <QString>
#include <memory>
#include "../types/Types.h"

namespace dle {

/**
 * @brief FileReader wraps QDataStream for reading Descent level files
 * 
 * Handles binary file reading with proper endianness for:
 * - RDL (Descent 1) - little-endian
 * - RL2 (Descent 2) - little-endian
 * - D2X-XL extensions
 */
class FileReader {
public:
    FileReader();
    ~FileReader();
    
    // Open file for reading
    bool open(const QString& filename);
    void close();
    bool isOpen() const;
    
    // Position
    qint64 pos() const;
    bool seek(qint64 pos);
    qint64 size() const;
    bool atEnd() const;
    
    // Read basic types
    int8_t readInt8();
    uint8_t readUInt8();
    int16_t readInt16();
    uint16_t readUInt16();
    int32_t readInt32();
    uint32_t readUInt32();
    
    // Read fixed-point
    fix readFix();
    
    // Read arrays
    void readBytes(char* data, qint64 len);
    QByteArray readBytes(qint64 len);
    QString readString(qint64 maxLen);
    
    // Read Descent types
    Vector readVector();
    Matrix readMatrix();
    UVLS readUVLS();
    
    // Skip bytes
    void skip(qint64 bytes);
    
    // Error handling
    bool hasError() const;
    QString errorString() const;

private:
    std::unique_ptr<QFile> m_file;
    std::unique_ptr<QDataStream> m_stream;
};

} // namespace dle

#endif // DLE_FILE_READER_H
