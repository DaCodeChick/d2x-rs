#include "FileWriter.h"

namespace dle {

FileWriter::FileWriter()
    : m_file(std::make_unique<QFile>())
    , m_stream(std::make_unique<QDataStream>())
{
}

FileWriter::~FileWriter() {
    close();
}

bool FileWriter::open(const QString& filename) {
    close();
    
    m_file = std::make_unique<QFile>(filename);
    if (!m_file->open(QIODevice::WriteOnly)) {
        return false;
    }
    
    m_stream = std::make_unique<QDataStream>(m_file.get());
    m_stream->setByteOrder(QDataStream::LittleEndian);  // Descent files are little-endian
    
    return true;
}

void FileWriter::close() {
    if (m_file && m_file->isOpen()) {
        m_file->close();
    }
}

bool FileWriter::isOpen() const {
    return m_file && m_file->isOpen();
}

qint64 FileWriter::pos() const {
    return m_file ? m_file->pos() : 0;
}

bool FileWriter::seek(qint64 pos) {
    return m_file && m_file->seek(pos);
}

// Write basic types
void FileWriter::writeInt8(int8_t value) {
    m_stream->writeRawData(reinterpret_cast<const char*>(&value), sizeof(value));
}

void FileWriter::writeUInt8(uint8_t value) {
    m_stream->writeRawData(reinterpret_cast<const char*>(&value), sizeof(value));
}

void FileWriter::writeInt16(int16_t value) {
    *m_stream << value;
}

void FileWriter::writeUInt16(uint16_t value) {
    *m_stream << value;
}

void FileWriter::writeInt32(int32_t value) {
    *m_stream << value;
}

void FileWriter::writeUInt32(uint32_t value) {
    *m_stream << value;
}

void FileWriter::writeFix(fix value) {
    writeInt32(value);  // Fixed-point is stored as int32
}

void FileWriter::writeBytes(const char* data, qint64 len) {
    m_stream->writeRawData(data, len);
}

void FileWriter::writeBytes(const QByteArray& data) {
    m_stream->writeRawData(data.data(), data.size());
}

void FileWriter::writeString(const QString& str, qint64 maxLen) {
    QByteArray data = str.toLatin1();
    
    // Truncate if too long
    if (data.size() > maxLen) {
        data.truncate(maxLen);
    }
    
    // Write string
    writeBytes(data.data(), data.size());
    
    // Pad with nulls if needed
    if (data.size() < maxLen) {
        writePadding(maxLen - data.size(), 0);
    }
}

void FileWriter::writeVector(const Vector& vec) {
    writeFix(vec.x);
    writeFix(vec.y);
    writeFix(vec.z);
}

void FileWriter::writeMatrix(const Matrix& mat) {
    writeVector(mat.right);
    writeVector(mat.up);
    writeVector(mat.forward);
}

void FileWriter::writeUVLS(const UVLS& uvls) {
    writeFix(uvls.u);
    writeFix(uvls.v);
    writeUInt16(uvls.light);
}

void FileWriter::writePadding(qint64 bytes, uint8_t value) {
    for (qint64 i = 0; i < bytes; ++i) {
        writeUInt8(value);
    }
}

bool FileWriter::hasError() const {
    return m_stream && m_stream->status() != QDataStream::Ok;
}

QString FileWriter::errorString() const {
    if (!m_file) {
        return "No file opened";
    }
    if (!m_file->isOpen()) {
        return "File not open";
    }
    if (hasError()) {
        switch (m_stream->status()) {
            case QDataStream::WriteFailed:
                return "Write failed";
            default:
                return "Unknown error";
        }
    }
    return m_file->errorString();
}

bool FileWriter::flush() {
    return m_file && m_file->flush();
}

} // namespace dle
